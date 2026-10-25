import { cancelConversion, convertFiles, type ConversionRequest } from "$lib/backend";
import { errorMessageOf } from "$lib/errors";
import type { Preferences } from "$lib/preferences";
import type { FileQueue } from "$lib/queue.svelte";
import { buildConversionRequests } from "$lib/requests";

// Starts, cancels and waits for a conversion of every file in the queue.
export class ConversionRun {
  isCancelling = $state(false);
  #activeBatchIds = $state<string[]>([]);
  #finished: Promise<void> = Promise.resolve();
  readonly #queue: FileQueue;

  constructor(queue: FileQueue) {
    this.#queue = queue;
  }

  async start(preferences: Preferences): Promise<void> {
    if (this.#queue.isEmpty || this.#queue.isConverting) {
      return;
    }

    this.#queue.prepareForRun();
    const requests = buildConversionRequests(this.#queue.items, preferences);
    this.#activeBatchIds = requests.map((request) => request.batchId);

    this.#finished = Promise.all(requests.map((request) => this.#runBatch(request))).then(() => {
      this.isCancelling = false;
    });
    await this.#finished;
  }

  async cancel(): Promise<void> {
    this.isCancelling = true;
    await Promise.all(this.#activeBatchIds.map((batchId) => cancelConversion(batchId)));
  }

  waitUntilFinished(): Promise<void> {
    return this.#finished;
  }

  // Images and videos run as separate batches; a failure only affects its own.
  async #runBatch(request: ConversionRequest): Promise<void> {
    try {
      await convertFiles(request, (event) => this.#queue.applyEvent(event));
    } catch (error) {
      const fileIds = request.files.map((file) => file.fileId);
      this.#queue.markUnfinishedAsFailed(fileIds, errorMessageOf(error));
    } finally {
      this.#activeBatchIds = this.#activeBatchIds.filter((batchId) => batchId !== request.batchId);
    }
  }
}
