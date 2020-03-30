import * as React from 'react'
import * as ReactDOM from 'react-dom'
import App from './App'
import './index.css'
import { unregister } from './registerServiceWorker'

declare global {
  interface Window {
    process_env: {
      SUBSTRATE_TELEMETRY_URL: string
      SUBSTRATE_TELEMETRY_SAMPLE: string
    }
  }
}
ReactDOM.render(<App />, document.getElementById('root') as HTMLElement)

unregister()
