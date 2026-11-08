#!/usr/bin/env python3
"""Teste de ponta a ponta do RID no emulador.

Controla a página pelo depurador da WebView (uma única conexão), tira prints
(base64 gzip entre PRINT_INICIO/PRINT_FIM) e imprime o texto da tela a cada passo.
"""
import base64
import gzip
import json
import subprocess
import sys
import time
import urllib.request

import websocket

PKG = "com.viniciuzdesign.rid"
CHAVE = "53261100000000000191650010000123451000123455"  # CNPJ do Banco do Brasil (Brasília)

AJUDANTES = r"""
window.__t = {
  botoes() { const d = document.querySelector('.dialogo'); return [...(d || document).querySelectorAll('button')]; },
  clicar(texto) {
    const b = this.botoes();
    const el = b.find(e => e.innerText.trim() === texto) || b.find(e => e.getAttribute('aria-label') === texto)
      || b.find(e => e.innerText.includes(texto));
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


def adb(*args, binario=False):
    r = subprocess.run(["adb", *args], capture_output=True)
    return r.stdout if binario else r.stdout.decode(errors="replace")


class Pagina:
    def __init__(self):
        self.ws = None
        self.ident = 0

    def conectar(self):
        for tentativa in range(30):
            sockets = [l.split()[-1].lstrip("@") for l in adb("shell", "cat", "/proc/net/unix").splitlines()
                       if "webview_devtools_remote" in l]
            if sockets:
                adb("forward", "tcp:9222", "localabstract:" + sockets[-1])
                try:
                    paginas = json.load(urllib.request.urlopen("http://127.0.0.1:9222/json", timeout=5))
                    if tentativa % 10 == 0:
                        print("páginas no depurador:", json.dumps(paginas)[:400])
                    url = next(p["webSocketDebuggerUrl"] for p in paginas
                               if p.get("type") == "page" and "webSocketDebuggerUrl" in p)
                    self.ws = websocket.create_connection(url, timeout=30, suppress_origin=True)
                    print("conectado ao depurador")
                    return
                except Exception as e:  # noqa: BLE001
                    print(f"aguardando depurador ({tentativa}): {e!r}")
            time.sleep(2)
        tirar_print("falha-sem-pagina")
        print("processo:", adb("shell", "pidof", PKG) or "(app fechado)")
        for linha in adb("logcat", "-d", "-v", "threadtime").splitlines():
            if (" E " in linha or " F " in linha or "AndroidRuntime" in linha) and \
                    ("rid" in linha.lower() or "AndroidRuntime" in linha or "Tauri" in linha or "chromium" in linha):
                print(linha[:240])
        print(adb("logcat", "-d", "-b", "crash"))
        raise SystemExit("Depurador da WebView não encontrado")

    def js(self, expressao):
        self.ident += 1
        self.ws.send(json.dumps({"id": self.ident, "method": "Runtime.evaluate",
                                 "params": {"expression": expressao, "returnByValue": True}}))
        while True:
            resposta = json.loads(self.ws.recv())
            if resposta.get("id") == self.ident:
                r = resposta.get("result", {})
                if "exceptionDetails" in r:
                    return "ERRO JS: " + json.dumps(r["exceptionDetails"])[:300]
                return r.get("result", {}).get("value")

    def t(self, chamada, espera=1.5):
        self.js(AJUDANTES)
        resultado = self.js("window.__t." + chamada)
        print("  ", resultado)
        time.sleep(espera)
        return resultado

    def tela(self):
        self.js(AJUDANTES)
        print("--- texto da tela ---")
        print(self.js("window.__t.texto()"))
        print("---------------------")


def passo(nome):
    print(f"\n=================== {nome} ===================", flush=True)


def tirar_print(nome):
    png = adb("exec-out", "screencap", "-p", binario=True)
    print(f"PRINT_INICIO {nome}")
    print(base64.b64encode(gzip.compress(png, 9)).decode())
    print(f"PRINT_FIM {nome}", flush=True)


def tocar_texto_nativo(texto):
    subprocess.run(["python3", "ci/ui.py", "tap", texto])


def main():
    apk = sys.argv[1]
    passo("Instalando")
    print(adb("install", "-r", apk))
    adb("logcat", "-c")
    adb("shell", "monkey", "-p", PKG, "-c", "android.intent.category.LAUNCHER", "1")
    time.sleep(20)
    pid = adb("shell", "pidof", PKG).strip()
    print("PID", pid)

    def ciclo_de_vida():
        for linha in adb("logcat", "-d", "-v", "time").splitlines():
            if PKG in linha and any(k in linha for k in ("ActivityTaskManager", "ActivityManager", "wm_", "Displayed")):
                print("   ", linha[:200])

    passo("Ciclo de vida da tela na abertura")
    ciclo_de_vida()
    print(adb("shell", "dumpsys", "activity", "activities").count(PKG + "/.MainActivity"), "referências à MainActivity")
    erro_inicial = None

    p = Pagina()
    try:
        p.conectar()
        p.js(AJUDANTES)
        erro_inicial = p.js("document.querySelector('.erro') ? document.querySelector('.erro').innerText : ''")
    except SystemExit:
        erro_inicial = "sem página no depurador"
    if erro_inicial:
        print("FALHA NA ABERTURA:", erro_inicial)
        print("Reabrindo o app uma vez...")
        adb("shell", "am", "force-stop", PKG)
        time.sleep(2)
        adb("logcat", "-c")
        adb("shell", "monkey", "-p", PKG, "-c", "android.intent.category.LAUNCHER", "1")
        time.sleep(20)
        pid = adb("shell", "pidof", PKG).strip()
        ciclo_de_vida()
        p = Pagina()
        p.conectar()
        p.js(AJUDANTES)
        print("Erro depois de reabrir:", repr(p.js("document.querySelector('.erro') ? document.querySelector('.erro').innerText : ''")))

    passo("1. Configurações")
    p.t('preencher("Nome completo", "Teste Diagnostico")', 0.3)
    p.t('preencher("CPF", "52998224725")', 0.3)
    p.t('preencher("Chave Pix", "teste@exemplo.com")', 0.3)
    p.t('clicar("Salvar")', 3)
    p.t('clicar("Voltar")', 3)
    p.tela()

    passo("2. Lançamento manual sem chave")
    p.t('clicar("Manual")', 2)
    tirar_print("02a-form-manual")
    p.t('preencher("Cidade", "sao paulo")', 0.3)
    p.t('preencher("Valor", "32,50")', 0.3)
    p.t('clicar("Salvar")', 3)
    p.tela()

    passo("3. Data futura deve ser bloqueada")
    p.t('clicar("Manual")', 2)
    p.t('preencher("Data do cupom", "2099-01-01")', 0.3)
    p.t('preencher("Cidade", "TESTE")', 0.3)
    p.t('preencher("Valor", "5")', 0.3)
    p.t('clicar("Salvar")', 2)
    p.tela()
    p.t('clicar("Cancelar")', 2)

    passo("4. Manual com chave (cidade automática) e data de 9 dias atrás")
    antes = p.js("(()=>{const d=new Date(Date.now()-9*864e5);"
                 "return d.getFullYear()+'-'+String(d.getMonth()+1).padStart(2,'0')+'-'+String(d.getDate()).padStart(2,'0')})()")
    print("data usada:", antes)
    p.t('clicar("Manual")', 2)
    p.t(f'preencher("Data do cupom", "{antes}")', 0.3)
    p.t(f'preencher("Chave NF-e", "{CHAVE}")', 1)
    inicio = time.time()
    cidade = ""
    while time.time() - inicio < 30 and not cidade:
        time.sleep(1)
        p.js(AJUDANTES)
        cidade = p.js('window.__t.valor("Cidade")')
    print(f"Cidade preenchida automaticamente: {cidade!r} em {time.time() - inicio:.1f}s")
    tirar_print("04a-cidade-automatica")
    p.t('preencher("Valor", "28,00")', 0.3)
    p.t('clicar("Salvar")', 3)
    p.tela()
    tirar_print("04b-lista-semanal")

    passo("5. Chave repetida deve avisar")
    p.t('clicar("Manual")', 2)
    p.t(f'preencher("Chave NF-e", "{CHAVE}")', 1)
    p.t('preencher("Cidade", "brasilia")', 0.3)
    p.t('preencher("Valor", "10")', 0.3)
    p.t('clicar("Salvar")', 3)
    p.tela()
    tirar_print("05a-aviso-repetida")
    p.t('clicar("Cancelar")', 1)
    p.t('clicar("Cancelar")', 2)

    passo("6. Editar o primeiro item (valor e cidade)")
    p.t('item(0)', 3)
    p.tela()
    tirar_print("06a-edicao")
    p.t('preencher("Valor", "40")', 0.3)
    p.t('preencher("Cidade", "campinas")', 0.3)
    p.t('clicar("Salvar")', 3)
    p.tela()

    passo("6b. Visual: modo claro e modo escuro")
    tirar_print("v1-inicio-claro")
    p.t('clicar("Opções")', 2)
    tirar_print("v2-menu-claro")
    p.t('clicar("Configurações")', 2)
    p.t('clicar("Escuro")', 2)
    tirar_print("v3-config-escuro")
    p.t('clicar("Voltar")', 3)
    tirar_print("v4-inicio-escuro")
    p.t('item(1)', 3)
    tirar_print("v5-edicao-escuro")
    p.t('clicar("Excluir lançamento")', 2)
    tirar_print("v6-folha-escuro")
    p.t('clicar("Cancelar")', 1)
    p.t('clicar("Cancelar")', 2)
    p.t('clicar("Opções")', 2)
    p.t('clicar("Configurações")', 2)
    p.t('clicar("Igual ao celular")', 2)
    p.t('clicar("Voltar")', 3)
    p.t('item(1)', 3)
    tirar_print("v7-edicao-claro")
    p.t('clicar("Cancelar")', 2)

    passo("7. Ler chave pelo QR Code na edição e cancelar")
    p.t('item(0)', 3)
    p.t('clicar("Ler chave pelo QR Code")', 6)
    tocar_texto_nativo("While using the app")
    time.sleep(6)
    tirar_print("07a-camera-na-edicao")
    p.t('clicar("Cancelar")', 3)
    p.tela()
    p.t('clicar("Cancelar")', 2)

    passo("8. Exportar relatório (início = pendente mais antigo)")
    p.t('clicar("Opções")', 2)
    p.t('clicar("Exportar relatório")', 3)
    print("Data inicial sugerida:", p.js('window.__t.valor("Data inicial")'))
    p.t('clicar("Copiar relatório")', 3)
    p.tela()
    tirar_print("08a-exportado")
    p.t('clicar("OK")', 1)
    print("Área de transferência do Android:")
    print(adb("shell", "cmd", "clipboard", "get-primary-clip") or "(comando indisponível neste Android)")
    p.t('clicar("Voltar")', 2)

    passo("9. Arquivar por período")
    p.t('clicar("Opções")', 2)
    p.t('clicar("Arquivar por período")', 3)
    p.t('clicar("Continuar")', 3)
    p.tela()
    p.t('clicar("Arquivar")', 3)
    p.tela()

    passo("10. Exportar sem pendentes deve avisar")
    p.t('clicar("Opções")', 2)
    p.t('clicar("Exportar relatório")', 3)
    p.t('preencher("Data inicial", "2026-01-01")', 0.3)
    p.t('clicar("Copiar relatório")', 3)
    p.tela()
    p.t('clicar("OK")', 1)
    p.t('clicar("Voltar")', 2)

    passo("11. Aba Arquivados e desarquivar um lançamento")
    p.t('clicar("Opções")', 2)
    p.t('clicar("Ver arquivados")', 3)
    p.tela()
    tirar_print("11a-arquivados")
    p.t('item(0)', 3)
    p.tela()
    p.t('clicar("Desarquivar")', 3)
    p.tela()

    passo("12. Arquivar de novo e desarquivar por período")
    p.t('clicar("Voltar")', 2)
    p.t('clicar("Opções")', 2)
    p.t('clicar("Arquivar por período")', 3)
    p.t('preencher("Data inicial", "2026-01-01")', 0.3)
    p.t('clicar("Continuar")', 3)
    p.t('clicar("Arquivar")', 3)
    p.t('clicar("Opções")', 2)
    p.t('clicar("Ver arquivados")', 3)
    p.t('clicar("Desarquivar período")', 3)
    p.t('preencher("Data inicial", "2026-01-01")', 0.3)
    p.t('clicar("Continuar")', 3)
    p.tela()
    p.t('clicar("Desarquivar")', 3)
    p.tela()
    p.t('clicar("Voltar")', 2)
    p.tela()

    passo("13. Excluir um lançamento")
    p.t('item(0)', 3)
    p.t('clicar("Excluir lançamento")', 2)
    p.t('clicar("Excluir")', 3)
    p.tela()

    passo("14. Excluir por período")
    p.t('clicar("Opções")', 2)
    p.t('clicar("Excluir por período")', 2)
    p.t('preencher("Data inicial", "2026-01-01")', 0.3)
    p.t('clicar("Continuar")', 3)
    p.tela()
    p.t('clicar("Excluir")', 3)
    p.tela()

    passo("O app ainda está aberto?")
    print("SIM, processo vivo" if adb("shell", "pidof", PKG).strip() else "NAO, o app fechou")

    passo("Erros no log do app")
    for linha in adb("logcat", "-d", "-v", "threadtime").splitlines():
        partes = linha.split()
        if len(partes) > 4 and partes[2] == pid and partes[4] in ("E", "F") \
                and "devtools" not in linha and "EGL" not in linha:
            print(linha[:220])

    passo("Crashes")
    print(adb("logcat", "-d", "-b", "crash"))
    passo("Fim")


if __name__ == "__main__":
    main()
