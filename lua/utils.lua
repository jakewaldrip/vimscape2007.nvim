local config = require("config")

---@class Utils
---@field round function round to the nearest whole number
---@field print_to_buffer function print the content to the buffer, accepts table of lines
---@field notify function log a notification gated by the configured log_level
local M = {}

---@return number
M.round = function(float)
	return math.floor(float + 0.5)
end

--- Log a notification if the given level meets the configured log_level threshold.
---@param msg string The message to display
---@param level integer A vim.log.levels value (e.g. vim.log.levels.INFO)
M.notify = function(msg, level)
	if level >= config.log_level then
		vim.notify(msg, level)
	end
end

M.print_to_buffer = function(content, bufnr)
	for k, v in pairs(content) do
		vim.api.nvim_buf_set_lines(bufnr, k, k, false, { v })
	end
end

return M
