import * as React from 'react'

import './ConnectionInput.css'
import { setTelemetryURL } from 'src/Connection'
function ConnectionInput() {
  const [connectionURI, setConnectionURI] = React.useState('')
  const [error, setError] = React.useState('')
  function handleChange(e) {
    setError('')
    const val = e.target.value
    setConnectionURI(val)
  }
  function handleSubmit(e) {
    e.preventDefault()
    const matcher = /^(ws|wss)/g

    const matches = connectionURI.match(matcher)

    if (!matches || matches.length !== 1) {
      setError('Please check your URL')
    } else {
      setTelemetryURL(connectionURI)
    }
  }
  return (
    <div className="ConnectionInput-connection">
      <div>
        <input
          name="connectionURL"
          placeholder="ws://YOUR_IP:YOUR_PORT/feed"
          type="text"
          onChange={handleChange}
          defaultValue={connectionURI}
        />
        <button onClick={handleSubmit}>save</button>
      </div>
      <div>
        <span className="ConnectionInput-error">{error}</span>
      </div>
    </div>
  )
}

export default ConnectionInput
