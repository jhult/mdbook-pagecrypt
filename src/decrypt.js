const data = `data:;base64,{{ encrypted }}`
const STORAGE_KEY = 'hashedPassword'

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

async function decrypt(nonce, cipherText) {
    const subtle = window.crypto.subtle || window.crypto.webkitSubtle

    const hashedPassword = loadPassword()
    if (!hashedPassword) {
        throw Error('Password expired or not found. Please refresh the page.')
    }
    const key = JSON.parse(hashedPassword)
    const derivedKey = await subtle.importKey('jwk', key, 'AES-GCM', true, ['decrypt'])
    const decrypted = await subtle.decrypt({ name: 'AES-GCM', iv: nonce }, derivedKey, cipherText)
    const decoded = new TextDecoder().decode(decrypted)
    console.log(decoded)
    return decoded
}

let onload = () => {}
if (document.currentScript) {
    // Defer onload invocation until the script is loaded.
    const oldOnload = document.currentScript.onload
    document.currentScript.onload = function (ev) {
        onload = () => oldOnload.call(this, ev)
    }
}

fetch(data)
    .then(response => response.arrayBuffer())
    .then(encrypted => {
        const nonce = encrypted.slice(32, 44)
        const cipherText = encrypted.slice(44)
        return decrypt(nonce, cipherText)
    })
    .then(decoded => {
        console.log("Decrypted JS with saved password")
        eval?.(decoded)
        onload()
    })
    .catch(error => console.error(error))
