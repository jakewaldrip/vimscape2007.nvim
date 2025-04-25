local utils = require("utils")

---@class WindowConfig
---@field vimscape_stats_bufnr integer Buffer to show the stats window inside
---@field stat_window_config table Config for the stats window
---@field vimscape_details_bufnr integer Buffer to show the details window inside
---@field details_window_config table Config for the details window
--
local M = {}

local ui = vim.api.nvim_list_uis()[1]
local width = utils.round(ui.width)
local height = utils.round(ui.height)

M.vimscape_stats_bufnr = vim.api.nvim_create_buf(false, true)

M.stat_window_config = {
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

M.vimscape_details_bufnr = vim.api.nvim_create_buf(false, true)

M.details_window_config = {
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

return M
