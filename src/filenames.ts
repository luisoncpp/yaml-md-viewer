export function defaultHtmlPath(sourcePath: string): string {
  return sourcePath.replace(/\.yaml\.md$/i, ".html");
}
