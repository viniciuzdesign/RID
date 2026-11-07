#!/usr/bin/env bash
# Reproduz no emulador: configurações -> Lançar cupom -> permissão da câmera,
# tira prints de cada passo e imprime o log do processo do app.
set -u
APK="$1"
PKG="com.viniciuzdesign.rid"
UI="python3 ci/ui.py"

passo() { echo; echo "=================== $1 ==================="; }
tecla() { adb shell input keyevent "$1"; sleep 0.7; }
tab() { tecla 61; }
enter() { tecla 66; }
digitar() { adb shell input text "$1"; sleep 0.7; }
print() {
  echo "PRINT_INICIO $1"
  adb exec-out screencap -p | gzip -9 | base64 -w0
  echo
  echo "PRINT_FIM $1"
}

passo "Instalando"
adb install -r "$APK"
adb shell wm size
adb logcat -c

passo "Abrindo o app"
adb shell monkey -p "$PKG" -c android.intent.category.LAUNCHER 1 >/dev/null
sleep 20
PID=$(adb shell pidof "$PKG")
echo "PID=$PID"
print 01-abertura

passo "Preenchendo configurações pelo teclado"
adb shell input tap 540 1500
sleep 1
tab; tab; digitar "Teste%sDiagnostico"
tab; digitar "52998224725"
tab; digitar "teste@exemplo.com"
tab; enter
sleep 4
print 02-salvo
tab; enter
sleep 4
print 03-pendentes

passo "Tocando em Lançar cupom"
tab; tab; enter
sleep 8
print 04-depois-de-lancar
$UI dump

passo "Permitindo a câmera (se o Android pedir)"
$UI tap "While using the app"
sleep 10
print 05-depois-da-permissao
$UI dump

passo "O app ainda está aberto?"
adb shell pidof "$PKG" && echo "SIM, processo vivo" || echo "NAO, o app fechou"

passo "Log do processo do app"
adb logcat -d -v threadtime | awk -v pid="$PID" '$3 == pid' | grep -v "EGL_emulation\|eglCodecCommon\|HostConnection\|gralloc" | tail -300

passo "Crashes"
adb logcat -d -b crash -v threadtime | tail -80

passo "Fim"
