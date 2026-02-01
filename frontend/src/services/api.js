const API_URL = import.meta.env.VITE_RAILS_API_URL || 'http://localhost:3000/api'

export async function apiFetch(path, { uid, method = 'GET', body } = {}) {
  const headers = {}

  if (uid) {
    headers['X-UID'] = uid
  }

  if (body) {
    headers['Content-Type'] = 'application/json'
  }

  const response = await fetch(`${API_URL}${path}`, {
    method,
    headers,
    body: body ? JSON.stringify(body) : undefined,
  })

  if (!response.ok) {
    const message = await response.text()
    throw new Error(message || `Request failed: ${response.status}`)
  }

  if (response.status === 204) {
    return null
  }

  return response.json()
}
