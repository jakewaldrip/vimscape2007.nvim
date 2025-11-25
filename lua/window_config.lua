local utils = require("utils")

---@class WindowConfig
---@field vimscape_stats_bufnr integer Buffer to show the stats window inside
---@field stat_window_config function Returns config for the stats window
---@field vimscape_details_bufnr integer Buffer to show the details window inside
---@field details_window_config function Returns config for the details window
--
local M = {}

local function get_ui_size()
	local ui = vim.api.nvim_list_uis()[1]
	return utils.round(ui.width), utils.round(ui.height)
end

M.vimscape_stats_bufnr = vim.api.nvim_create_buf(false, true)

M.stat_window_config = function()
	local width, height = get_ui_size()
	return {
		relative = "editor",
		width = utils.round(width / 2),
		height = utils.round(height / 2),
		col = utils.round(width / 4),
		row = utils.round(height / 4),
		style = "minimal",
		focusable = true,
		border = "double",
		title = "Skills",
		title_pos = "center",
		footer = "[q]uit | [d]etails",
	}
end

M.vimscape_details_bufnr = vim.api.nvim_create_buf(false, true)

M.details_window_config = function()
	return {
		relative = "cursor",
		width = 24,
		height = 4,
		col = 0,
		row = 0,
		style = "minimal",
		focusable = false,
		border = "rounded",
		-- title = "Details",
		zindex = 99,
	}
end

return M
