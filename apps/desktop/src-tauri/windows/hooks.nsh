!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "Adding Shehata Git command-line tools to the current user PATH"
  ExecWait '"$INSTDIR\shehata.exe" path install "$INSTDIR"' $0
  ${If} $0 != 0
    DetailPrint "Warning: Shehata Git could not update the current user PATH (exit code $0)"
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DetailPrint "Removing Shehata Git command-line tools from the current user PATH"
  ExecWait '"$INSTDIR\shehata.exe" path uninstall "$INSTDIR"' $0
  ${If} $0 != 0
    DetailPrint "Warning: Shehata Git could not clean the current user PATH (exit code $0)"
  ${EndIf}
!macroend
