local function get_plugin_dir()
	local str = debug.getinfo(1, "S").source:sub(2) -- Remove the "@" character
	return vim.fn.fnamemodify(str, ":p:h:h") -- Get the directory name, go one level up
end

---@class Config
---@field db_path string [required] The path to the vimscape database
---@field batch_size integer The number of keys typed before processing the batch
---@field log_level integer Minimum log level for notifications (vim.log.levels)
---@field batch_notify boolean Whether to notify on batch processing
local M = {
	db_path = get_plugin_dir() .. "/",
	batch_size = 50,
	log_level = vim.log.levels.INFO,
	batch_notify = false,
}

return M
