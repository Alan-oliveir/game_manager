//pub fn get_tool_status(app: &AppHandle, tool: ManagedTool) -> Result<ToolStatus, AppError> {
// 1. tenta cache em app_config (get_config)
// 2. valida que o path cacheado ainda existe no disco
// 3. se inválido/ausente: busca no PATH (crate `which`)
// 4. se não achou: busca em ~/.local/share/playlite/tools/<bin_name>
// 5. se achou (2, 3 ou 4): roda --version, salva no app_config, retorna Found
// 6. senão: retorna NotFound (sem erro — é um estado válido)
//}