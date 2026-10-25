// Stands in for the Rust backend while the UI tests run in a plain browser.
// tests/ui/vite.config.ts loads this before the app, and it answers the
// app's Tauri calls through @tauri-apps/api's own IPC mocks.
import type { Channel, InvokeArgs } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import type { OpenDialogOptions } from "@tauri-apps/plugin-dialog";
import type {
  ConversionCatalog,
  ConversionEvent,
  ConversionRequest,
  InspectedFile,
  MediaKind,
  PathInspection,
} from "$lib/backend";
import { buildCatalog, classifyFileName } from "./catalog";
import type {
  BackendLog,
  DragEventType,
  FileConversionEvent,
  MockBackendControls,
  MockScenario,
  SavedSettings,
} from "./scenario";

// How Rust delivers channel messages: numbered, then an end marker.
type ChannelMessage = { index: number; message: ConversionEvent } | { index: number; end: true };

interface TauriInternals {
  runCallback(callbackId: number, data: ChannelMessage): void;
  convertFileSrc(filePath: string): string;
}

declare global {
  interface Window {
    __TAURI_INTERNALS__: TauriInternals;
  }
}

interface ActiveBatch {
  request: ConversionRequest;
  channelId: number;
  nextMessageIndex: number;
  unfinishedFileIds: Set<string>;
  resolve: () => void;
  reject: (message: string) => void;
}

type DialogResult = string[] | string | null;

const CLOSE_REQUESTED_EVENT = "tauri://close-requested";
const QUIT_REQUESTED_EVENT = "quit-requested";
// Like the backend, one add stops at this many files (MAX_INSPECTED_FILES in input_files.rs).
const MAX_INSPECTED_FILES = 10_000;
const DRAG_EVENT_NAMES: Record<DragEventType, string> = {
  enter: "tauri://drag-enter",
  drop: "tauri://drag-drop",
  leave: "tauri://drag-leave",
};
const FINISHED_EVENT_TYPES: ReadonlySet<ConversionEvent["type"]> = new Set(["completed", "failed", "cancelled"]);
const SAMPLE_SIZES_BYTES: Record<MediaKind | "unsupported", number> = {
  image: 2_400_000,
  video: 48_000_000,
  unsupported: 12_000,
};
const AUTOMATIC_PROGRESS_PERCENT = 50;
const STORE_RESOURCE_ID = 1;

function readScenario(): MockScenario {
  const scenario = window.converterScenario;
  if (!scenario) {
    throw new Error("No mock scenario. Start the app with launchConverter().");
  }
  return scenario;
}

const scenario = readScenario();
const pendingDialogResults: DialogResult[] = [];
const heldScans: (() => void)[] = [];
let isHoldingScans = false;
let releaseHeldCatalog: () => void = () => {};
const catalogReleased = new Promise<void>((resolve) => {
  releaseHeldCatalog = resolve;
});
const activeBatches: ActiveBatch[] = [];
const backendLog: BackendLog = {
  dialogs: [],
  conversionRequests: [],
  cancelledBatchIds: [],
  openedOutputs: [],
  revealedOutputs: [],
  openedUrls: [],
  windowDestroys: [],
  storedPreferences: scenario.savedPreferences,
};

function fileNameOf(path: string): string {
  const fileName = path.slice(path.lastIndexOf("/") + 1);
  return fileName;
}

function folderOf(path: string): string {
  const folder = path.slice(0, path.lastIndexOf("/"));
  return folder;
}

function stemOf(fileName: string): string {
  const dotIndex = fileName.lastIndexOf(".");
  const stem = dotIndex === -1 ? fileName : fileName.slice(0, dotIndex);
  return stem;
}

// Like the real backend, a folder adds only the convertible files inside it.
function expandFolder(path: string): string[] {
  const folderContents = scenario.folders[path];
  if (!folderContents) {
    return [path];
  }

  const convertiblePaths = folderContents.filter((filePath) => classifyFileName(fileNameOf(filePath)) !== null);
  return convertiblePaths;
}

function inspectFile(path: string): InspectedFile {
  const name = fileNameOf(path);
  const kind = classifyFileName(name);

  const inspectedFile: InspectedFile = { path, name, sizeBytes: SAMPLE_SIZES_BYTES[kind ?? "unsupported"], kind };
  return inspectedFile;
}

function inspectPaths(paths: string[]): PathInspection {
  const foundPaths = paths.flatMap(expandFolder);
  const inspection: PathInspection = {
    files: foundPaths.slice(0, MAX_INSPECTED_FILES).map(inspectFile),
    isFileLimitReached: foundPaths.length > MAX_INSPECTED_FILES,
  };
  return inspection;
}

function inspectPathsWhenReleased(paths: string[]): PathInspection | Promise<PathInspection> {
  if (!isHoldingScans) {
    return inspectPaths(paths);
  }

  const scanFinished = new Promise<PathInspection>((resolve) => heldScans.push(() => resolve(inspectPaths(paths))));
  return scanFinished;
}

function releaseScans(): void {
  isHoldingScans = false;
  for (const finishScan of heldScans.splice(0)) {
    finishScan();
  }
}

async function getCatalog(): Promise<ConversionCatalog> {
  if (scenario.holdsCatalog) {
    await catalogReleased;
  }
  const catalog = buildCatalog(scenario.platform);
  return catalog;
}

function outputPathFor(inputPath: string, request: ConversionRequest): string {
  const outputDirectory = request.outputDirectory ?? folderOf(inputPath);
  const outputPath = `${outputDirectory}/${stemOf(fileNameOf(inputPath))}.${request.settings.format}`;
  return outputPath;
}

function sendChannelMessage(batch: ActiveBatch, content: { message: ConversionEvent } | { end: true }): void {
  const channelMessage: ChannelMessage = { index: batch.nextMessageIndex, ...content };
  batch.nextMessageIndex += 1;
  window.__TAURI_INTERNALS__.runCallback(batch.channelId, channelMessage);
}

function endBatch(batch: ActiveBatch): void {
  sendChannelMessage(batch, { end: true });
  activeBatches.splice(activeBatches.indexOf(batch), 1);
}

function finishBatch(batch: ActiveBatch): void {
  endBatch(batch);
  batch.resolve();
}

function findBatch(kind: MediaKind): ActiveBatch {
  const batch = activeBatches.find((activeBatch) => activeBatch.request.settings.kind === kind);
  if (!batch) {
    throw new Error(`No ${kind} batch is converting`);
  }
  return batch;
}

// Rust returns errors as plain strings, so the promise rejects with one.
function failBatch(kind: MediaKind, message: string): void {
  const batch = findBatch(kind);
  endBatch(batch);
  batch.reject(message);
}

function endBatchEarly(kind: MediaKind): void {
  finishBatch(findBatch(kind));
}

// convert_files resolves once every file in its batch has finished, as in Rust.
function deliverEvent(batch: ActiveBatch, event: ConversionEvent): void {
  sendChannelMessage(batch, { message: event });
  if (!FINISHED_EVENT_TYPES.has(event.type)) {
    return;
  }

  batch.unfinishedFileIds.delete(event.fileId);
  if (batch.unfinishedFileIds.size === 0) {
    finishBatch(batch);
  }
}

function convertFileAutomatically(batch: ActiveBatch, file: ConversionRequest["files"][number]): void {
  const fileId = file.fileId;
  deliverEvent(batch, { type: "started", fileId });

  // Only videos report progress; images convert in one step.
  if (batch.request.settings.kind === "video") {
    deliverEvent(batch, { type: "progress", fileId, percent: AUTOMATIC_PROGRESS_PERCENT });
  }

  const failureMessage = scenario.failingFiles[fileNameOf(file.path)];
  const finishedEvent: ConversionEvent = failureMessage
    ? { type: "failed", fileId, message: failureMessage }
    : { type: "completed", fileId, outputPath: outputPathFor(file.path, batch.request) };
  deliverEvent(batch, finishedEvent);
}

function convertBatchAutomatically(batch: ActiveBatch): void {
  for (const file of batch.request.files) {
    convertFileAutomatically(batch, file);
  }
}

type BatchSettlers = Pick<ActiveBatch, "resolve" | "reject">;

function startBatch(request: ConversionRequest, channel: Channel<ConversionEvent>, settlers: BatchSettlers): void {
  const batch: ActiveBatch = {
    request,
    channelId: channel.id,
    nextMessageIndex: 0,
    unfinishedFileIds: new Set(request.files.map((file) => file.fileId)),
    ...settlers,
  };
  activeBatches.push(batch);

  if (scenario.conversionMode === "automatic") {
    setTimeout(() => convertBatchAutomatically(batch));
  }
}

function convertFiles(request: ConversionRequest, channel: Channel<ConversionEvent>): Promise<void> {
  backendLog.conversionRequests.push(request);

  const batchFinished = new Promise<void>((resolve, reject) => startBatch(request, channel, { resolve, reject }));
  return batchFinished;
}

function cancelUnfinishedFiles(batch: ActiveBatch): void {
  for (const fileId of [...batch.unfinishedFileIds]) {
    deliverEvent(batch, { type: "cancelled", fileId });
  }
}

// Like Rust, cancelling only signals the batch; its files stop a moment later.
function cancelConversion(batchId: string): void {
  backendLog.cancelledBatchIds.push(batchId);
  const batch = activeBatches.find((activeBatch) => activeBatch.request.batchId === batchId);
  if (!batch) {
    return;
  }

  setTimeout(() => cancelUnfinishedFiles(batch));
}

function findConvertingFile(fileName: string): { batch: ActiveBatch; fileId: string } {
  for (const batch of activeBatches) {
    const file = batch.request.files.find((requestedFile) => fileNameOf(requestedFile.path) === fileName);
    if (file) {
      return { batch, fileId: file.fileId };
    }
  }
  throw new Error(`${fileName} is not being converted`);
}

function sendFileEvent(fileName: string, event: FileConversionEvent): void {
  const { batch, fileId } = findConvertingFile(fileName);
  const conversionEvent: ConversionEvent = { ...event, fileId };
  deliverEvent(batch, conversionEvent);
}

function openDialog(options: OpenDialogOptions): DialogResult {
  backendLog.dialogs.push(options);
  const dialogResult = pendingDialogResults.shift() ?? null;
  return dialogResult;
}

function storeValue(value: SavedSettings): void {
  backendLog.storedPreferences = value;
}

function readStoredPreferences(): [SavedSettings | null, boolean] {
  const storedPreferences = backendLog.storedPreferences;
  return [storedPreferences, storedPreferences !== null];
}

// The app's typed wrappers in src/lib/backend.ts and the plugins decide
// these shapes, so they are trusted here.
function argumentsAs<CommandArguments>(payload: InvokeArgs | undefined): CommandArguments {
  const commandArguments = payload as CommandArguments;
  return commandArguments;
}

type CommandHandler = (payload: InvokeArgs | undefined) => unknown;

const COMMAND_HANDLERS: Record<string, CommandHandler> = {
  get_catalog: () => getCatalog(),
  inspect_input_paths: (payload) => inspectPathsWhenReleased(argumentsAs<{ paths: string[] }>(payload).paths),
  convert_files: (payload) => {
    const { request, onEvent } = argumentsAs<{ request: ConversionRequest; onEvent: Channel<ConversionEvent> }>(payload);
    return convertFiles(request, onEvent);
  },
  cancel_conversion: (payload) => cancelConversion(argumentsAs<{ batchId: string }>(payload).batchId),
  open_output: (payload) => {
    backendLog.openedOutputs.push(argumentsAs<{ path: string }>(payload).path);
  },
  reveal_output: (payload) => {
    backendLog.revealedOutputs.push(argumentsAs<{ path: string }>(payload).path);
  },
  "plugin:window|destroy": () => {
    backendLog.windowDestroys.push({ runningBatchCount: activeBatches.length });
  },
  "plugin:opener|open_url": (payload) => {
    backendLog.openedUrls.push(argumentsAs<{ url: string }>(payload).url);
  },
  "plugin:dialog|open": (payload) => openDialog(argumentsAs<{ options: OpenDialogOptions }>(payload).options),
  "plugin:store|load": () => STORE_RESOURCE_ID,
  "plugin:store|get": () => readStoredPreferences(),
  "plugin:store|set": (payload) => storeValue(argumentsAs<{ value: SavedSettings }>(payload).value),
};

function handleCommand(command: string, payload: InvokeArgs | undefined): unknown {
  // Like Rust, a failing command rejects with a plain string.
  const failureMessage = scenario.failingCommands[command];
  if (failureMessage !== undefined) {
    return Promise.reject(failureMessage);
  }

  const commandHandler = COMMAND_HANDLERS[command];
  if (!commandHandler) {
    throw new Error(`The mock backend has no handler for ${command}`);
  }

  const result = commandHandler(payload);
  return result;
}

// Real previews come from the asset protocol, which a browser can't load.
function placeholderPreviewUrl(filePath: string): string {
  const hue = [...filePath].reduce((sum, character) => sum + character.charCodeAt(0), 0) % 360;
  const svg =
    `<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64">` +
    `<linearGradient id="fill" x2="1" y2="1"><stop stop-color="hsl(${hue} 65% 62%)"/>` +
    `<stop offset="1" stop-color="hsl(${hue + 50} 55% 38%)"/></linearGradient>` +
    `<rect width="64" height="64" fill="url(#fill)"/></svg>`;

  const previewUrl = `data:image/svg+xml,${encodeURIComponent(svg)}`;
  return previewUrl;
}

const controls: MockBackendControls = {
  setNextDialogResult: (dialogResult) => {
    pendingDialogResults.push(dialogResult);
  },
  emitDragEvent: (type, paths) => emit(DRAG_EVENT_NAMES[type], { paths, position: { x: 0, y: 0 } }),
  sendFileEvent,
  failBatch,
  endBatchEarly,
  holdScans: () => {
    isHoldingScans = true;
  },
  releaseScans,
  releaseCatalog: () => releaseHeldCatalog(),
  requestWindowClose: () => emit(CLOSE_REQUESTED_EVENT, null),
  requestQuitFromMenu: () => emit(QUIT_REQUESTED_EVENT, null),
  readLog: () => backendLog,
};

mockWindows("main");
mockIPC(handleCommand, { shouldMockEvents: true });
window.__TAURI_INTERNALS__.convertFileSrc = placeholderPreviewUrl;
window.converterMockBackend = controls;
