// Writes THIRD-PARTY-NOTICES.md: the bundled FFmpeg, every npm package built into the UI and
// every Rust crate compiled into the app, with their licences.
//
//   pnpm notices
import { execFileSync } from "node:child_process";
import { existsSync, readdirSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import path from "node:path";
import { build, type Plugin } from "vite";
import { FFMPEG_BUILDS, FFMPEG_SOURCE, FFMPEG_SOURCE_ARCHIVE_NAME, FFMPEG_VERSION } from "./ffmpeg-builds.ts";

const ROOT_DIRECTORY = path.join(import.meta.dirname, "..");
const NOTICES_PATH = path.join(ROOT_DIRECTORY, "THIRD-PARTY-NOTICES.md");
const REPOSITORY_URL = "https://github.com/Squ1ggly/converter-desktop";
const LICENCE_FILE_PATTERN = /^(licen[cs]e|copying|copyright|notice)/i;

interface ThirdPartyPackage {
  name: string;
  version: string;
  licence: string;
  homepage: string;
  directory: string;
}

interface PackageJson {
  name: string;
  version: string;
  license?: string;
  homepage?: string;
  repository?: string | { url?: string };
}

interface CargoPackage {
  id: string;
  name: string;
  version: string;
  license: string | null;
  repository: string | null;
  homepage: string | null;
  manifest_path: string;
}

interface CargoDependency {
  pkg: string;
  dep_kinds: { kind: string | null }[];
}

interface CargoMetadata {
  packages: CargoPackage[];
  resolve: { root: string; nodes: { id: string; deps: CargoDependency[] }[] };
}

function readLicenceTexts(directory: string): string[] {
  const licenceFileNames = readdirSync(directory)
    .filter((fileName) => LICENCE_FILE_PATTERN.test(fileName))
    .sort();
  const licenceTexts = licenceFileNames.map((fileName) =>
    readFileSync(path.join(directory, fileName), "utf8").replaceAll("\r\n", "\n").trim(),
  );
  return licenceTexts;
}

// "/a/node_modules/.pnpm/x@1/node_modules/@scope/name/dist/x.js" -> "/a/node_modules/.pnpm/x@1/node_modules/@scope/name"
function packageDirectoryOf(moduleId: string): string | undefined {
  const normalisedId = moduleId.replaceAll("\\", "/").replace(/^\0/, "").split("?")[0] ?? "";
  const marker = "/node_modules/";
  const markerIndex = normalisedId.lastIndexOf(marker);
  if (markerIndex === -1) {
    return undefined;
  }

  const pathSegments = normalisedId.slice(markerIndex + marker.length).split("/");
  const nameSegmentCount = pathSegments[0]?.startsWith("@") ? 2 : 1;
  const packageName = pathSegments.slice(0, nameSegmentCount).join("/");
  const packageDirectory = normalisedId.slice(0, markerIndex + marker.length) + packageName;
  return packageDirectory;
}

// Turns "github:owner/name", "owner/name" and "git+https://host/owner/name.git" into a browsable URL.
function homepageOf(packageJson: PackageJson): string {
  if (packageJson.homepage) {
    return packageJson.homepage;
  }

  const repository = typeof packageJson.repository === "string" ? packageJson.repository : packageJson.repository?.url;
  if (!repository) {
    return `https://www.npmjs.com/package/${packageJson.name}`;
  }

  const shorthand = repository.match(/^(?:github:)?([\w.-]+\/[\w.-]+)$/);
  const repositoryUrl = shorthand ? `https://github.com/${shorthand[1]}` : repository.replace(/^git\+/, "").replace(/\.git$/, "");
  return repositoryUrl;
}

// Tailwind inlines the packages a stylesheet imports, so they never reach the module graph.
function listCssPackageDirectories(): string[] {
  const stylesheet = readFileSync(path.join(ROOT_DIRECTORY, "src", "app.css"), "utf8");
  const specifiers = [...stylesheet.matchAll(/^@(?:import|plugin) "([^".][^"]*)"/gm)].map((match) => match[1] ?? "");
  const packageDirectories = specifiers.map((specifier) => {
    const nameSegmentCount = specifier.startsWith("@") ? 2 : 1;
    const packageName = specifier.split("/").slice(0, nameSegmentCount).join("/");
    return path.join(ROOT_DIRECTORY, "node_modules", packageName);
  });
  return packageDirectories;
}

// The packages the UI build reads, CSS and fonts included. Some of them, like Svelte itself, are
// devDependencies, so `pnpm licenses list --prod` would miss them.
async function listBundledNpmPackages(): Promise<ThirdPartyPackage[]> {
  const moduleIds: string[] = [];
  const collectModuleIds: Plugin = {
    name: "collect-module-ids",
    generateBundle(_options, bundle) {
      const assetSources = Object.values(bundle).flatMap((item) => (item.type === "asset" ? item.originalFileNames : []));
      moduleIds.push(...this.getModuleIds(), ...assetSources);
    },
  };

  await build({
    configFile: path.join(ROOT_DIRECTORY, "vite.config.ts"),
    logLevel: "warn",
    plugins: [collectModuleIds],
    build: { write: false },
  });
  const bundledDirectories = moduleIds.map(packageDirectoryOf).filter((directory) => directory !== undefined);
  const packageDirectories = new Set([...bundledDirectories, ...listCssPackageDirectories()].map((directory) => realpathSync(directory)));

  const packages = [...packageDirectories].map((directory) => {
    const packageJson = JSON.parse(readFileSync(path.join(directory, "package.json"), "utf8")) as PackageJson;
    const thirdPartyPackage: ThirdPartyPackage = {
      name: packageJson.name,
      version: packageJson.version,
      licence: packageJson.license ?? "See licence text",
      homepage: homepageOf(packageJson),
      directory,
    };
    return thirdPartyPackage;
  });
  return packages;
}

// Crates the app links on the platforms it ships for. Build and dev dependencies aren't shipped.
function listLinkedCrates(): ThirdPartyPackage[] {
  const platformFilters = Object.keys(FFMPEG_BUILDS).flatMap((target) => ["--filter-platform", target]);
  const manifestPath = path.join(ROOT_DIRECTORY, "src-tauri", "Cargo.toml");
  const metadataJson = execFileSync(
    "cargo",
    ["metadata", "--format-version", "1", "--locked", "--manifest-path", manifestPath, ...platformFilters],
    { encoding: "utf8", maxBuffer: 256 * 1024 * 1024 },
  );
  const metadata = JSON.parse(metadataJson) as CargoMetadata;
  const packagesById = new Map(metadata.packages.map((cargoPackage) => [cargoPackage.id, cargoPackage]));
  const nodesById = new Map(metadata.resolve.nodes.map((node) => [node.id, node]));

  const linkedIds = new Set<string>();
  const pendingIds = [metadata.resolve.root];
  while (pendingIds.length > 0) {
    const node = nodesById.get(pendingIds.pop() ?? "");
    const normalDependencies = (node?.deps ?? []).filter((dependency) =>
      dependency.dep_kinds.some((dependencyKind) => dependencyKind.kind === null),
    );
    const unseenIds = normalDependencies.map((dependency) => dependency.pkg).filter((id) => !linkedIds.has(id));
    unseenIds.forEach((id) => linkedIds.add(id));
    pendingIds.push(...unseenIds);
  }

  const crates = [...linkedIds].map((id) => {
    const cargoPackage = packagesById.get(id);
    if (!cargoPackage) {
      throw new Error(`cargo metadata lists ${id} as a dependency but not as a package`);
    }

    const crate: ThirdPartyPackage = {
      name: cargoPackage.name,
      version: cargoPackage.version,
      licence: cargoPackage.license ?? "See licence text",
      homepage: cargoPackage.repository ?? cargoPackage.homepage ?? `https://crates.io/crates/${cargoPackage.name}`,
      directory: path.dirname(cargoPackage.manifest_path),
    };
    return crate;
  });
  return crates;
}

function byNameAndVersion(first: ThirdPartyPackage, second: ThirdPartyPackage): number {
  const comparison = first.name.localeCompare(second.name, "en") || first.version.localeCompare(second.version, "en");
  return comparison;
}

function packageTable(packages: ThirdPartyPackage[]): string {
  const rows = packages.map(
    (thirdPartyPackage) =>
      `| [${thirdPartyPackage.name}](${thirdPartyPackage.homepage}) | ${thirdPartyPackage.version} | ${thirdPartyPackage.licence} |`,
  );
  const table = ["| Package | Version | Licence |", "|---|---|---|", ...rows].join("\n");
  return table;
}

function ffmpegSection(): string {
  const buildRows = Object.entries(FFMPEG_BUILDS).map(
    ([target, ffmpegBuild]) =>
      `| ${ffmpegBuild.platformName} | [${ffmpegBuild.builder.name}](${ffmpegBuild.builder.buildScriptsUrl}) | \`ffmpeg-buildconf-${target}.txt\` |`,
  );

  const section = `## FFmpeg

Converter converts files by running [FFmpeg](https://ffmpeg.org) ${FFMPEG_VERSION}, which is installed
next to the app as a separate program named \`converter-ffmpeg\`. FFmpeg is licensed under the GNU
General Public License, version 3 or later. The installers include the licence as
\`licenses/FFmpeg/COPYING.GPLv3\` and FFmpeg's own licensing notes as \`licenses/FFmpeg/LICENSE.md\`.

| Platform | Built by | Build configuration |
|---|---|---|
${buildRows.join("\n")}

### Source code

Every [Converter release](${REPOSITORY_URL}/releases) has these files attached next to its
installers:

- \`${FFMPEG_SOURCE_ARCHIVE_NAME}\`: the FFmpeg ${FFMPEG_VERSION} source, as published at ${FFMPEG_SOURCE.url}
- \`ffmpeg-buildconf-<target>.txt\`: the configuration each build was made with, as printed by
  \`converter-ffmpeg -buildconf\`

The builds also include libraries such as x264, x265, libvpx, libaom, libwebp, LAME, Opus and
Vorbis. The build scripts linked above list the version and source of each one.

### Written offer

For at least three years after each release, the source above stays attached to that release. If
it is ever missing, [open an issue](${REPOSITORY_URL}/issues) and Squ1ggly will provide the
complete corresponding source of the FFmpeg in that release, charging no more than the cost of
providing it.`;
  return section;
}

function licenceTextsSection(packages: ThirdPartyPackage[]): string {
  const packageNamesByText = new Map<string, string[]>();
  const packagesWithoutText: ThirdPartyPackage[] = [];
  for (const thirdPartyPackage of packages) {
    const licenceTexts = readLicenceTexts(thirdPartyPackage.directory);
    if (licenceTexts.length === 0) {
      packagesWithoutText.push(thirdPartyPackage);
    }

    for (const licenceText of licenceTexts) {
      const packageNames = packageNamesByText.get(licenceText) ?? [];
      packageNames.push(`${thirdPartyPackage.name} ${thirdPartyPackage.version}`);
      packageNamesByText.set(licenceText, packageNames);
    }
  }

  const entries = [...packageNamesByText.entries()]
    .map(([licenceText, packageNames]) => ({ licenceText, heading: [...new Set(packageNames)].sort().join(", ") }))
    .sort((first, second) => first.heading.localeCompare(second.heading, "en"));
  const blocks = entries.map((entry) => `### ${entry.heading}\n\n\`\`\`\`text\n${entry.licenceText}\n\`\`\`\``);
  const namesWithoutText = packagesWithoutText.map((thirdPartyPackage) => `${thirdPartyPackage.name} ${thirdPartyPackage.version}`);
  const section = `## Licence texts

These packages don't include a licence file, so their licence is the standard text of the
[SPDX licence](https://spdx.org/licenses/) named in the tables above: ${namesWithoutText.join(", ")}.

${blocks.join("\n\n")}`;
  return section;
}

async function main(): Promise<void> {
  const npmPackages = (await listBundledNpmPackages()).sort(byNameAndVersion);
  const crates = listLinkedCrates().sort(byNameAndVersion);

  const notices = `# Third-party notices

Converter is licensed under the [MIT License](LICENSE). Its installers also contain the
third-party software below, under their own licences.

This file is generated by \`pnpm notices\`. Don't edit it by hand.

${ffmpegSection()}

## User interface packages

These npm packages are built into the app's user interface.

${packageTable(npmPackages)}

## Rust crates

These crates are compiled into the app.

${packageTable(crates)}

${licenceTextsSection([...npmPackages, ...crates])}
`;

  writeFileSync(NOTICES_PATH, notices);
  console.log(`Wrote ${path.relative(process.cwd(), NOTICES_PATH)}: ${npmPackages.length} npm packages, ${crates.length} crates`);
}

if (!existsSync(path.join(ROOT_DIRECTORY, "node_modules"))) {
  throw new Error("Run `pnpm install` first");
}

await main();
