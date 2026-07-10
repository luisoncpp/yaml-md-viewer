export interface DocumentView { revision: number; sourcePath: string; displayTitle: string; compiledHtml: string }
export interface AppError { code: string; message: string }
export interface SaveResult { path: string }
