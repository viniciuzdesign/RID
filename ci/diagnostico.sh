#!/usr/bin/env bash
# Reproduz no emulador: configurações -> Lançar cupom -> permissão da câmera,
# e imprime o log do Android com os erros.
set -u
APK="$1"
PKG="com.viniciuzdesign.rid"
UI="python3 ci/ui.py"

passo() { echo; echo "=================== $1 ==================="; }

passo "Instalando"
adb install -r "$APK"
adb logcat -c

passo "Abrindo o app"
adb shell monkey -p "$PKG" -c android.intent.category.LAUNCHER 1 >/dev/null
sleep 20
$UI dump

passo "Preenchendo configurações"
$UI edit 0 "Teste Diagnostico"
$UI edit 1 "52998224725"
$UI edit 2 "teste@exemplo.com"
adb shell input keyevent 111
sleep 1
$UI tap "Salvar"
sleep 4
$UI dump
$UI tap "Voltar"
sleep 4
$UI dump

passo "Tocando em Lançar cupom"
$UI tap "Lançar cupom"
sleep 8
$UI dump

passo "Permitindo a câmera (se o Android pedir)"
$UI tap "While using the app" || true
sleep 1
$UI tap "Allow" || true
sleep 10
$UI dump

passo "O app ainda está aberto?"
adb shell pidof "$PKG" && echo "SIM, processo vivo" || echo "NAO, o app fechou"

passo "Log do Android (erros e Tauri)"
adb logcat -d -v brief | grep -E "AndroidRuntime|FATAL|Exception|Error|Tauri|RustStdoutStderr|chromium|BarcodeScanner|$PKG|at app\.|at com\.|Caused by" | grep -v "ResourceType\|EGL_emulation\|GoogleApiManager\|Finsky\|gms" | tail -400

passo "Fim"
