#!/usr/bin/env python3
"""Controla a página do app no emulador pelo protocolo de depuração do Chrome (só no teste).

Uso: cdp.py '<expressão JS>'   (as funções de window.__t ficam disponíveis)
"""
import json
import subprocess
import sys
import time
import urllib.request

import websocket

AJUDANTES = r"""
window.__t = window.__t || {
  botoes() { const d = document.querySelector('.dialogo'); return [...(d || document).querySelectorAll('button')]; },
  clicar(texto) {
    const b = this.botoes();
    const el = b.find(e => e.innerText.trim() === texto) || b.find(e => e.innerText.includes(texto));
    if (!el) return 'NAO ACHOU botão: ' + texto;
    el.click(); return 'clicou: ' + texto;
  },
  campo(rotulo) {
    const lab = [...document.querySelectorAll('label')].find(l => l.innerText.trim().startsWith(rotulo));
    return lab && lab.querySelector('input,textarea');
  },
  preencher(rotulo, valor) {
    const el = this.campo(rotulo);
    if (!el) return 'NAO ACHOU campo: ' + rotulo;
    el.value = valor;
    el.dispatchEvent(new Event('input', { bubbles: true }));
    el.dispatchEvent(new Event('change', { bubbles: true }));
    return 'preencheu ' + rotulo + ' = ' + valor;
  },
  valor(rotulo) { const el = this.campo(rotulo); return el ? el.value : 'NAO ACHOU campo: ' + rotulo; },
  item(i) {
    const it = document.querySelectorAll('button.item');
    if (!it[i]) return 'NAO ACHOU item ' + i;
    it[i].click(); return 'abriu item ' + i;
  },
  texto() { return document.body.innerText; },
};
'ok'
"""


def adb(*args):
    return subprocess.run(["adb", *args], capture_output=True, text=True).stdout


def conectar():
    for _ in range(20):
        sockets = [l.split()[-1].lstrip("@") for l in adb("shell", "cat", "/proc/net/unix").splitlines()
                   if "webview_devtools_remote" in l]
        if sockets:
            adb("forward", "tcp:9222", "localabstract:" + sockets[-1])
            try:
                paginas = json.load(urllib.request.urlopen("http://127.0.0.1:9222/json", timeout=5))
                pagina = next(p for p in paginas if p.get("type") == "page")
                return websocket.create_connection(pagina["webSocketDebuggerUrl"], timeout=20)
            except Exception as e:  # noqa: BLE001
                print("aguardando depurador:", e, file=sys.stderr)
        time.sleep(2)
    raise SystemExit("Depurador da WebView não encontrado")


def avaliar(ws, expressao, ident=[0]):
    ident[0] += 1
    ws.send(json.dumps({"id": ident[0], "method": "Runtime.evaluate",
                        "params": {"expression": expressao, "returnByValue": True, "awaitPromise": True}}))
    while True:
        resposta = json.loads(ws.recv())
        if resposta.get("id") == ident[0]:
            r = resposta.get("result", {})
            if "exceptionDetails" in r:
                return "ERRO JS: " + json.dumps(r["exceptionDetails"])[:500]
            return r.get("result", {}).get("value")


def main():
    ws = conectar()
    avaliar(ws, AJUDANTES)
    resultado = avaliar(ws, "window.__t." + sys.argv[1] if not sys.argv[1].startswith("(") else sys.argv[1])
    print(resultado)


if __name__ == "__main__":
    main()
