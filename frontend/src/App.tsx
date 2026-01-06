import { useState, useEffect } from 'react'
import { LoginButton, useAuth, api } from './auth'

interface Item {
  id: string
  name: string
  description?: string
  created_at: string
  updated_at: string
}

interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
}

function App() {
  const { isAuthenticated, user } = useAuth()
  const [items, setItems] = useState<Item[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [newItemName, setNewItemName] = useState('')
  const [newItemDescription, setNewItemDescription] = useState('')
  const [health, setHealth] = useState<{ status: string; version: string } | null>(null)

  useEffect(() => {
    api('/health')
      .then((res) => res.json())
      .then((data: ApiResponse<{ status: string; version: string }>) => {
        if (data.success && data.data) setHealth(data.data)
      })
      .catch((err) => console.error('Health check failed:', err))
  }, [])

  useEffect(() => { fetchItems() }, [])

  async function fetchItems() {
    try {
      setLoading(true)
      const res = await api('/items')
      const data: ApiResponse<{ items: Item[]; count: number }> = await res.json()
      if (data.success && data.data) setItems(data.data.items)
      else setError(data.error || 'Failed to fetch items')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch items')
    } finally {
      setLoading(false)
    }
  }

  async function createItem(e: React.FormEvent) {
    e.preventDefault()
    if (!newItemName.trim()) return
    try {
      const res = await api('/items', {
        method: 'POST',
        body: JSON.stringify({ name: newItemName, description: newItemDescription || undefined }),
      })
      const data: ApiResponse<Item> = await res.json()
      if (data.success && data.data) {
        setItems([...items, data.data])
        setNewItemName('')
        setNewItemDescription('')
      } else setError(data.error || 'Failed to create item')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create item')
    }
  }

  async function deleteItem(id: string) {
    try {
      await api(`/items/${id}`, { method: 'DELETE' })
      setItems(items.filter((item) => item.id !== id))
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete item')
    }
  }

  return (
    <div className="min-h-screen bg-gray-100 py-8">
      <div className="max-w-2xl mx-auto px-4">
        {/* Header with auth */}
        <div className="bg-white rounded-lg shadow-md p-6 mb-6">
          <div className="flex justify-between items-center mb-2">
            <h1 className="text-2xl font-bold text-gray-800">My App</h1>
            <LoginButton />
          </div>
          {health && <p className="text-sm text-green-600">API: {health.status} (v{health.version})</p>}
          {isAuthenticated && user && (
            <p className="text-sm text-blue-600 mt-1">Welcome, {user.name || user.email}!</p>
          )}
        </div>

        {error && (
          <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-6">
            {error}
            <button onClick={() => setError(null)} className="float-right font-bold">×</button>
          </div>
        )}

        <div className="bg-white rounded-lg shadow-md p-6 mb-6">
          <h2 className="text-lg font-semibold mb-4">Create Item</h2>
          <form onSubmit={createItem} className="space-y-4">
            <input type="text" placeholder="Item name" value={newItemName} onChange={(e) => setNewItemName(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500" />
            <input type="text" placeholder="Description (optional)" value={newItemDescription} onChange={(e) => setNewItemDescription(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500" />
            <button type="submit" className="w-full bg-blue-500 text-white py-2 px-4 rounded-md hover:bg-blue-600 transition">Add Item</button>
          </form>
        </div>

        <div className="bg-white rounded-lg shadow-md p-6">
          <h2 className="text-lg font-semibold mb-4">Items {!loading && `(${items.length})`}</h2>
          {loading ? <p className="text-gray-500">Loading...</p> : items.length === 0 ? (
            <p className="text-gray-500">No items yet. Create one above!</p>
          ) : (
            <ul className="space-y-3">
              {items.map((item) => (
                <li key={item.id} className="flex justify-between items-start p-3 bg-gray-50 rounded-md">
                  <div>
                    <p className="font-medium text-gray-800">{item.name}</p>
                    {item.description && <p className="text-sm text-gray-600">{item.description}</p>}
                    <p className="text-xs text-gray-400 mt-1">Created: {new Date(item.created_at).toLocaleString()}</p>
                  </div>
                  <button onClick={() => deleteItem(item.id)} className="text-red-500 hover:text-red-700 text-sm">Delete</button>
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>
    </div>
  )
}

export default App
