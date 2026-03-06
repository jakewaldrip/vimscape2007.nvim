---@class Config
---@field db_path string The directory path for the vimscape database
---@field db_name string The filename for the database file
---@field batch_size integer The number of keys typed before processing the batch
---@field log_level integer Minimum log level for notifications (vim.log.levels)
---@field token_log boolean Whether to enable token logging to file for integration testing
---@field recording_on boolean Whether recording is on by default when the plugin starts
---@field key_overrides table<string, string> Map of physical keys to substituted keys for the lexer (e.g. { [";"] = ":" })
local M = {
  db_path = vim.fn.stdpath("data") .. "/vimscape2007/",
  db_name = "vimscape.db",
  batch_size = 1000,
  log_level = vim.log.levels.INFO,
  token_log = false,
  key_overrides = {},
  recording_on = true,
}

return M
