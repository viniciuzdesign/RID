#!/usr/bin/env python3
"""Automação simples via adb + uiautomator para o teste de diagnóstico.

Uso:
  ui.py dump                 -> imprime os textos visíveis na tela
  ui.py tap "<texto>"        -> toca no primeiro elemento cujo texto contém <texto>
  ui.py edit <n> "<valor>"   -> toca no n-ésimo campo de texto (0, 1, 2...) e digita o valor
"""
import re
import subprocess
import sys
import time
import xml.etree.ElementTree as ET


def adb(*args):
    return subprocess.run(["adb", *args], capture_output=True, text=True).stdout


def nodes():
    for _ in range(5):
        adb("shell", "uiautomator", "dump", "/sdcard/ui.xml")
        xml = adb("shell", "cat", "/sdcard/ui.xml")
        if "<hierarchy" in xml:
            return list(ET.fromstring(xml[xml.index("<hierarchy"):]).iter("node"))
        time.sleep(2)
    return []


def centro(node):
    x1, y1, x2, y2 = map(int, re.findall(r"\d+", node.get("bounds")))
    return (x1 + x2) // 2, (y1 + y2) // 2


def texto(node):
    return (node.get("text") or "") + (node.get("content-desc") or "")


def main():
    acao = sys.argv[1]
    if acao == "dump":
        for n in nodes():
            t = texto(n).strip()
            if t:
                print(f"  [{n.get('class')}] {t}")
        return
    if acao == "tap":
        alvo = sys.argv[2].lower()
        lista = nodes()
        exatos = [n for n in lista if texto(n).strip().lower() == alvo]
        parciais = [n for n in lista if alvo in texto(n).lower()]
        for n in exatos + parciais:
                x, y = centro(n)
                adb("shell", "input", "tap", str(x), str(y))
                print(f"tocou em '{texto(n)}' ({x},{y})")
                return
        print(f"NAO ENCONTRADO: {sys.argv[2]}")
        return
    if acao == "edit":
        indice, valor = int(sys.argv[2]), sys.argv[3]
        campos = [n for n in nodes() if n.get("class") == "android.widget.EditText"]
        if indice >= len(campos):
            print(f"campo {indice} nao encontrado ({len(campos)} campos)")
            return
        x, y = centro(campos[indice])
        adb("shell", "input", "tap", str(x), str(y))
        time.sleep(1)
        adb("shell", "input", "text", valor.replace(" ", "%s"))
        print(f"digitou no campo {indice}")


if __name__ == "__main__":
    main()
