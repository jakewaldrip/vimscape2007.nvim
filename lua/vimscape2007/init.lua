local keys = require("keys")
local utils = require("utils")
local globals = require("globals")
local window_config = require("window_config")
local vimscape = require("vimscape_backend")
local config = require("config")

local ns = vim.api.nvim_create_namespace("vimscape_keys")

---@class Vimscape2007
---@field setup function Setup the plugin. Sets up commands, config, and inits database
---@field toggle function Toggles recording
---@field show_data function Opens a window relative buffer that displays your stats
---@field show_details function Opens a cursor relative buffer that displays details about the stat your cursor is on
---@field create_user_commands function Creates the user command for interacting with vimscape
local M = {}

---@param opts Config?
M.setup = function(opts)
	vim.notify("Ran setup for Vimscape", vim.log.levels.INFO)

	config = vim.tbl_deep_extend("force", config, opts or {})
	vimscape.setup_tables(config.db_path)

	M.create_user_commands()
end

M.toggle = function()
	globals.active = not globals.active

	if globals.active then
		vim.notify("Vimscape recording started", vim.log.levels.INFO, {})
	else
		vim.notify("Vimscape recording paused", vim.log.levels.INFO, {})
		vim.api.nvim_buf_clear_namespace(0, ns, 0, -1)
		globals.typed_letters = {}
	end
end

M.show_data = function()
	vim.keymap.set("n", "q", ":q<CR>", { silent = true, buffer = window_config.vimscape_stats_bufnr })
	vim.keymap.set(
		"n",
		"d",
		"<cmd>Vimscape details<CR>",
		{ silent = true, buffer = window_config.vimscape_stats_bufnr }
	)

	local bufr_width = window_config.stat_window_config.width
	local user_data = vimscape.get_user_data(bufr_width, config.db_path)

	vim.api.nvim_open_win(window_config.vimscape_stats_bufnr, true, window_config.stat_window_config)
	vim.api.nvim_buf_set_lines(window_config.vimscape_stats_bufnr, 0, -1, false, {})
	utils.print_to_buffer(user_data, window_config.vimscape_stats_bufnr)

	-- See below
	-- vim.bo[window_config.vimscape_stats_bufnr].modifiable = false
end

M.show_details = function(word)
	vim.keymap.set("n", "q", ":q<CR>", { silent = true, buffer = window_config.vimscape_details_bufnr })

	local details_data = vimscape.get_skill_details(word, config.db_path)

	window_config.details_window_config["title"] = word
	vim.api.nvim_open_win(window_config.vimscape_details_bufnr, true, window_config.details_window_config)
	vim.api.nvim_buf_set_lines(window_config.vimscape_details_bufnr, 0, -1, false, {})
	utils.print_to_buffer(details_data, window_config.vimscape_details_bufnr)

	-- Issue: Can't open a second time since it's not modifiable
	-- Solution possibly to remake buffer every time, and define it locally?
	-- vim.bo[window_config.vimscape_details_bufnr].modifiable = false
end

-- Where the magic happens
vim.on_key(keys.record_keys, ns)

M.create_user_commands = function()
	vim.api.nvim_create_user_command("Vimscape", function(cmd_opts)
		local command = cmd_opts.args

		-- Show details
		if command == "details" then
			local word = vim.fn.expand("<cword>")
			M.show_details(word)
		end

		-- Show stats
		if command == "stats" then
			M.show_data()
		end

		-- Toggle recording
		if command == "toggle" then
			M.toggle()
		end
	end, { nargs = 1 })
end

return M
