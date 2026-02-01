const ALPHABET = '0123456789ABCDEFGHJKMNPQRSTVWXYZ'

export function generateUid(length = 12) {
  const bytes = new Uint8Array(length)
  crypto.getRandomValues(bytes)

  const raw = Array.from(bytes, (byte) => ALPHABET[byte % ALPHABET.length]).join('')
  return raw.match(/.{1,4}/g).join('-')
}
