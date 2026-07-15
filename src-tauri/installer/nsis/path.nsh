!macro AddToUserPath
  ReadRegStr $0 HKCU "Environment" "PATH"
  WriteRegExpandStr HKCU "Environment" "PATH" "$0;$INSTDIR"
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
!macroend

!macro RemoveFromUserPath
  ReadRegStr $0 HKCU "Environment" "PATH"
  Push $0
  Push "$INSTDIR;"
  Call StrStr
  Pop $1
  StrCmp $1 "" +3
    StrCpy $0 $0 ""
    StrCpy $0 $0 $1 0
  Push "$INSTDIR"
  Push $0
  Call StrStr
  Pop $1
  StrCmp $1 "" +3
    StrCpy $0 $0 ""
    StrCpy $0 $0 $1 0
  WriteRegExpandStr HKCU "Environment" "PATH" "$0"
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
!macroend

Function StrStr
  Exch $R0
  Exch
  Exch $R1
  Push $R2
  Push $R3
  Push $R4
  Push $R5
  StrLen $R2 $R0
  StrCpy $R4 0
  loop:
    StrCpy $R3 $R1 $R2 $R4
    StrCmp $R3 $R0 found
    StrCmp $R3 "" done
    IntOp $R4 $R4 + 1
    Goto loop
  found:
    StrCpy $R5 $R1 "" $R4
  done:
    StrCpy $R0 $R5
    Pop $R5
    Pop $R4
    Pop $R3
    Pop $R2
    Pop $R1
    Pop $R1
    Exch $R0
FunctionEnd

Section "-AddAmusToPath"
  !insertmacro AddToUserPath
SectionEnd

Section "Un.RemoveAmusFromPath"
  !insertmacro RemoveFromUserPath
SectionEnd
