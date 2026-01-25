#!/usr/bin/env -S deno run --allow-read --allow-write --allow-run --no-lock

import {parseArgs} from "jsr:@std/cli@1.0.13"
import {stringify} from "jsr:@libs/xml@7.0.3"
import remarkHeadingShift from "npm:remark-heading-shift@1.1.2"
import remarkParse from "npm:remark-parse@11.0.0"
import remarkStringify from "npm:remark-stringify@11.0.0"
import {unified} from "npm:unified@11.0.5"
import {dirname, extname, join} from "jsr:@std/path@1.1.4"

const args = parseArgs(Deno.args, {
  string: ["output"],
  alias: {
    output: "o",
  },
})

const rootUrl = new URL(".", import.meta.url)

const isMarkdownPath = (path: string) => path.toLowerCase().endsWith(".md")

const resolvePath = (path: string) => new URL(path, rootUrl)

const fileExists = async (path: string) => {
  try {
    await Deno.stat(resolvePath(path))
    return true
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) {
      return false
    }
    throw error
  }
}

const shiftHeadings = async (markdown: string) => {
  const file = await unified()
    .use(remarkParse)
    .use(remarkHeadingShift, 1)
    .use(remarkStringify)
    .process(markdown)
  return String(file).trimEnd()
}

const renderCodeFile = (path: string, contents: string) => {
  const identifier = getLanguageIdentifier(path)
  const trimmed = contents.trimEnd()
  const fence = getFence(trimmed)
  const fenced = `${fence}${identifier}\n${trimmed}\n${fence}`
  return [`### ${path}`, fenced].join("\n\n")
}

const getFence = (contents: string) => {
  const matches = contents.match(/`+/g) ?? []
  const max = matches.reduce((current, match) => Math.max(current, match.length), 0)
  return "`".repeat(Math.max(3, max + 1))
}

const getLanguageIdentifier = (path: string) => {
  const extension = extname(path)
  switch (extension) {
    case ".ts":
      return "typescript"
    case ".rs":
      return "rust"
    case ".xml":
      return "xml"
    case ".toml":
      return "toml"
    default:
      throw new Error(`Could not get a language identifier for extension: ${extension}`)
  }
}

const renderXmlFile = (path: string, contents: string) =>
  stringify(
    {
      file: {
        path,
        contents: "\n" + contents,
      },
    },
    {
      format: {
        indent: "",
        breakline: 0,
      },
    },
  ).trimEnd()

const includeFile = async (path: string) => {
  const contents = await Deno.readTextFile(resolvePath(path))
  return await renderFileContents(path, contents)
}

const renderFileContents = async (path: string, contents: string, pathToRender: string = path) => {
  if (isMarkdownPath(path)) {
    return await shiftHeadings(contents)
  }
  return renderCodeFile(pathToRender, contents)
}

type CargoMetadata = {
  packages: CargoPackage[]
  resolve: CargoResolve | null
}

type CargoResolve = {
  root: string | null
  nodes: CargoNode[]
}

type CargoNode = {
  id: string
  deps: CargoDependency[]
}

type CargoDependency = {
  name: string
  pkg: string
}

type CargoPackage = {
  id: string
  name: string
  version: string
  manifest_path: string
}

const loadCargoMetadata = (() => {
  let cached: Promise<CargoMetadata> | null = null
  const decoder = new TextDecoder()
  return () => {
    if (!cached) {
      cached = (async () => {
        const command = new Deno.Command("cargo", {
          args: ["metadata", "--format-version=1"],
          stdout: "piped",
          stderr: "piped",
        })
        const output = await command.output()
        if (!output.success) {
          const stderr = decoder.decode(output.stderr).trim()
          throw new Error(`cargo metadata failed${stderr ? `: ${stderr}` : ""}`)
        }
        return JSON.parse(decoder.decode(output.stdout)) as CargoMetadata
      })()
    }
    return cached
  }
})()

const includeCargoDependencyFile = async (dependencyName: string, path: string) => {
  const metadata = await loadCargoMetadata()
  const resolve = metadata.resolve
  if (!resolve?.root) {
    throw new Error("cargo metadata did not include a resolve root")
  }
  const rootNode = resolve.nodes.find((node) => node.id === resolve.root)
  if (!rootNode) {
    throw new Error(`cargo metadata did not include the root node: '${resolve.root}'`)
  }
  const dependency = rootNode.deps.find((dep) => dep.name === dependencyName)
  if (!dependency) {
    throw new Error(`cargo dependency not found: '${dependencyName}'`)
  }
  const cargoPackage = metadata.packages.find((pkg) => pkg.id === dependency.pkg)
  if (!cargoPackage) {
    throw new Error(`cargo package not found for dependency: '${dependencyName}'`)
  }
  const crateRoot = dirname(cargoPackage.manifest_path)
  const fullPath = join(crateRoot, path)
  const contents = await Deno.readTextFile(fullPath)
  return await renderFileContents(path, contents, `${dependencyName}/${path}`)
}

const includeFileIfExists = async (path: string) => {
  const exists = await fileExists(path)
  if (!exists) {
    return null
  }
  return await includeFile(path)
}

const parts = (await Promise.all([
  Promise.resolve("<!-- This file is autogenerated by AGENTS.ts -->"),
  Promise.resolve("# Guidelines"),
  includeFile(".agents/general.md"),
  includeFileIfExists(".agents/project.md"),
  includeFileIfExists(".agents/knowledge.md"),
  includeFileIfExists(".agents/gotchas.md"),
  includeCargoDependencyFile("errgonomic", "DOCS.md"),
  includeFile(".agents/cli.md"),
  Promise.resolve("## Project files"),
  includeFile("Cargo.toml"),
  includeFileIfExists("src/main.rs"),
  includeFileIfExists("src/lib.rs"),
])).filter((part): part is string => !!part && part.length > 0)

const content = parts.join("\n\n")

if (args.output) {
  // The file must be writable by the `agent` user in the sandbox (not read-only)
  await Deno.writeTextFile(args.output, `${content}\n`)
} else {
  console.info(content)
}
