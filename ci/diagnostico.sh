#!/usr/bin/env bash
# Teste de ponta a ponta no emulador. A página é controlada pelo depurador da WebView
# (ci/cdp.py); prints de cada passo vão em base64 gzip entre PRINT_INICIO/PRINT_FIM.
set -u
APK="$1"
PKG="com.viniciuzdesign.rid"
T="python3 ci/cdp.py"
UI="python3 ci/ui.py"
CHAVE="53261100000000000191650010000123451000123455"  # CNPJ do Banco do Brasil (Brasília)

passo() { echo; echo "=================== $1 ==================="; }
print() {
  echo "PRINT_INICIO $1"
  adb exec-out screencap -p | gzip -9 | base64 -w0
  echo
  echo "PRINT_FIM $1"
}
tela() { echo "--- texto da tela ---"; $T 'texto()'; echo "---------------------"; }
t() { $T "$1"; sleep "${2:-1.5}"; }

passo "Instalando"
adb install -r "$APK"
adb logcat -c
adb shell monkey -p "$PKG" -c android.intent.category.LAUNCHER 1 >/dev/null
sleep 20
PID=$(adb shell pidof "$PKG"); echo "PID=$PID"

passo "1. Configurações"
t 'preencher("Nome completo", "Teste Diagnostico")' 0.3
t 'preencher("CPF", "52998224725")' 0.3
t 'preencher("Chave Pix", "teste@exemplo.com")' 0.3
t 'clicar("Salvar")' 3
t 'clicar("Voltar")' 3
tela

passo "2. Lançamento manual sem chave"
t 'clicar("Manual")' 2
print 02a-form-manual
tela
t 'preencher("Cidade", "sao paulo")' 0.3
t 'preencher("Valor", "32,50")' 0.3
t 'clicar("Salvar")' 3
tela

passo "3. Data futura deve ser bloqueada"
t 'clicar("Manual")' 2
t 'preencher("Data do cupom", "2099-01-01")' 0.3
t 'preencher("Cidade", "TESTE")' 0.3
t 'preencher("Valor", "5")' 0.3
t 'clicar("Salvar")' 2
tela
t 'clicar("Cancelar")' 2

passo "4. Lançamento manual com chave (cidade automática) e data de ontem"
t 'clicar("Manual")' 2
ONTEM=$($T '(()=>{const d=new Date(Date.now()-864e5);return d.toISOString().slice(0,10)})()')
echo "ontem=$ONTEM"
t "preencher(\"Data do cupom\", \"$ONTEM\")" 0.3
t "preencher(\"Chave NF-e\", \"$CHAVE\")" 12
echo "Cidade preenchida: $($T 'valor("Cidade")')"
print 04a-cidade-automatica
t 'preencher("Valor", "28,00")' 0.3
t 'clicar("Salvar")' 3
tela
print 04b-lista-semanal

passo "5. Chave repetida deve avisar"
t 'clicar("Manual")' 2
t "preencher(\"Chave NF-e\", \"$CHAVE\")" 10
t 'preencher("Valor", "10")' 0.3
t 'clicar("Salvar")' 3
tela
print 05a-aviso-repetida
t 'clicar("Cancelar")' 1
t 'clicar("Cancelar")' 2

passo "6. Editar o primeiro item (valor e cidade)"
t 'item(0)' 3
tela
print 06a-edicao
t 'preencher("Valor", "40")' 0.3
t 'preencher("Cidade", "campinas")' 0.3
t 'clicar("Salvar")' 3
tela

passo "7. Ler chave pelo QR Code na edição e cancelar"
t 'item(0)' 3
t 'clicar("Ler chave pelo QR Code")' 6
$UI tap "While using the app"
sleep 6
print 07a-camera-na-edicao
t 'clicar("Cancelar")' 3
tela
t 'clicar("Cancelar")' 2

passo "8. Excluir um lançamento"
t 'item(0)' 3
t 'clicar("Excluir lançamento")' 2
tela
t 'clicar("Excluir")' 3
tela

passo "9. Excluir por período"
t 'clicar("⋮")' 2
t 'clicar("Excluir lançamentos por período")' 2
t "preencher(\"Data inicial\", \"$ONTEM\")" 0.3
t 'clicar("Continuar")' 3
tela
print 09a-confirmar-periodo
t 'clicar("Excluir")' 3
tela

passo "O app ainda está aberto?"
adb shell pidof "$PKG" && echo "SIM, processo vivo" || echo "NAO, o app fechou"

passo "Log do processo do app"
adb logcat -d -v threadtime | awk -v pid="$PID" '$3 == pid' \
  | grep -v "EGL_emulation\|eglCodecCommon\|HostConnection\|gralloc\|ComponentDiscovery\|DynamiteModule" | grep -E " [EWF] |FATAL|Exception" | tail -80

passo "Crashes"
adb logcat -d -b crash -v threadtime | tail -60

passo "Fim"
