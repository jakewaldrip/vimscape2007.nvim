local function get_plugin_dir()
  local str = debug.getinfo(1, "S").source:sub(2)
  return vim.fn.fnamemodify(str, ":p:h:h")
end

---@class Config
---@field db_path string The directory path for the vimscape database
---@field db_name string The filename for the database file
---@field batch_size integer The number of keys typed before processing the batch
---@field log_level integer Minimum log level for notifications (vim.log.levels)
---@field batch_notify boolean Whether to notify on batch processing
---@field token_log boolean Whether to enable token logging to file for integration testing
---@field key_overrides table<string, string> Map of physical keys to substituted keys for the lexer (e.g. { [";"] = ":" })
local M = {
  db_path = get_plugin_dir() .. "/",
  db_name = "vimscape.db",
  batch_size = 1000,
  log_level = vim.log.levels.ERROR,
  batch_notify = false,
  token_log = false,
  key_overrides = {},
}

return M
