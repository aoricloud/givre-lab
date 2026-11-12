// Le module WebAssembly, chargé une fois.
let exportations = null;

export async function leModule() {
  if (exportations === null) {
    const octets = await fetch(new URL("../givre.wasm", import.meta.url)).then((reponse) => reponse.arrayBuffer());
    exportations = (await WebAssembly.instantiate(octets, {})).instance.exports;
  }
  return exportations;
}
