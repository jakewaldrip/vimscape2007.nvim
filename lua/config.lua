local function get_plugin_dir()
	local str = debug.getinfo(1, "S").source:sub(2) -- Remove the "@" character
	return vim.fn.fnamemodify(str, ":p:h:h") -- Get the directory name, go one level up
end

---@class Config
---@field db_path string [required] The path to the vimscape database
---@field batch_size integer The number of keys typed before processing the batch
local M = {
	db_path = get_plugin_dir() .. "/",
	batch_size = 50,
}

return M
