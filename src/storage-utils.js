const STORAGE_KEY = 'hashedPassword'
const EXPIRATION_MS = 7 * 24 * 60 * 60 * 1000

function savePassword(keyData) {
    const data = {
        key: keyData,
        expires: Date.now() + EXPIRATION_MS
    }
    localStorage.setItem(STORAGE_KEY, JSON.stringify(data))
}

function loadPassword() {
    try {
        const stored = localStorage.getItem(STORAGE_KEY)
        if (!stored) return null
        const data = JSON.parse(stored)
        if (Date.now() > data.expires) {
            localStorage.removeItem(STORAGE_KEY)
            return null
        }
        return JSON.stringify(data.key)
    } catch {
        return null
    }
}

function clearPassword() {
    localStorage.removeItem(STORAGE_KEY)
}
