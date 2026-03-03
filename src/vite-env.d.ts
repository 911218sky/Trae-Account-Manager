/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_NODE_ENV: string
  readonly VITE_LOG_LEVEL: string
  readonly VITE_LOG_FILE_ENABLED: string
  readonly VITE_LOG_FILE_PATH: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
