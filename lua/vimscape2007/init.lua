local keys = require("keys")
local utils = require("utils")
local globals = require("globals")
local window_config = require("window_config")
local vimscape = require("vimscape_backend")
local config = require("config")

local ns = vim.api.nvim_create_namespace("vimscape_keys")

---Returns the full path to the database file
---@return string
local function get_db_full_path()
	return config.db_path .. config.db_name
end

local record_key = function(key)
	keys.record_keys(key, get_db_full_path(), config.batch_size, config)
end

---@class Vimscape2007
---@field setup function Setup the plugin. Sets up commands, config, and inits database
---@field toggle function Toggles recording
---@field show_data function Opens a window relative buffer that displays your stats
---@field show_details function Opens a cursor relative buffer that displays details about the stat your cursor is on
---@field create_user_commands function Creates the user command for interacting with vimscape
local M = {}

---@param opts Config?
M.setup = function(opts)
	if vim.log.levels.INFO >= config.log_level then
		vim.notify("Ran setup for Vimscape", vim.log.levels.INFO)
	end

	config = vim.tbl_deep_extend("force", config, opts or {})
	vimscape.setup_tables(get_db_full_path())

	M.create_user_commands()
end

M.toggle = function()
	globals.set_active(not globals.get_active())

	if globals.get_active() then
		if vim.log.levels.INFO >= config.log_level then
			vim.notify("Vimscape recording started", vim.log.levels.INFO, {})
		end
		vim.on_key(record_key, ns)
	else
		if vim.log.levels.INFO >= config.log_level then
			vim.notify("Vimscape recording paused", vim.log.levels.INFO, {})
		end
		vim.on_key(nil, ns)
		vim.api.nvim_buf_clear_namespace(0, ns, 0, -1)
		globals.clear_typed_letters()
	end
end

M.show_data = function()
	if vim.api.nvim_buf_is_valid(window_config.vimscape_stats_bufnr) then
		vim.api.nvim_buf_delete(window_config.vimscape_stats_bufnr, { force = true })
	end
	window_config.vimscape_stats_bufnr = vim.api.nvim_create_buf(false, true)

	vim.keymap.set("n", "q", ":q<CR>", { silent = true, buffer = window_config.vimscape_stats_bufnr })
	vim.keymap.set(
		"n",
		"d",
		"<cmd>Vimscape details<CR>",
		{ silent = true, buffer = window_config.vimscape_stats_bufnr }
	)

	local stat_config = window_config.stat_window_config()
	local bufr_width = stat_config.width
	local user_data = vimscape.get_user_data(bufr_width, get_db_full_path())

	vim.api.nvim_open_win(window_config.vimscape_stats_bufnr, true, stat_config)
	vim.api.nvim_buf_set_lines(window_config.vimscape_stats_bufnr, 0, -1, false, {})
	utils.print_to_buffer(user_data, window_config.vimscape_stats_bufnr)

	vim.bo[window_config.vimscape_stats_bufnr].modifiable = false
end

M.show_details = function(word)
	if vim.api.nvim_buf_is_valid(window_config.vimscape_details_bufnr) then
		vim.api.nvim_buf_delete(window_config.vimscape_details_bufnr, { force = true })
	end
	window_config.vimscape_details_bufnr = vim.api.nvim_create_buf(false, true)

	vim.keymap.set("n", "q", ":q<CR>", { silent = true, buffer = window_config.vimscape_details_bufnr })

	local details_data = vimscape.get_skill_details(word, get_db_full_path())

	local details_config = window_config.details_window_config()
	details_config.title = word
	vim.api.nvim_open_win(window_config.vimscape_details_bufnr, true, details_config)
	vim.api.nvim_buf_set_lines(window_config.vimscape_details_bufnr, 0, -1, false, {})
	utils.print_to_buffer(details_data, window_config.vimscape_details_bufnr)

	vim.bo[window_config.vimscape_details_bufnr].modifiable = false
end

M.create_user_commands = function()
	vim.api.nvim_create_user_command("Vimscape", function(cmd_opts)
		local command = cmd_opts.args

		if command == "" then
			vim.notify("Vimscape commands: stats, details, toggle", vim.log.levels.INFO)
			return
		end

		-- Show details
		if command == "details" then
			local word = vim.fn.expand("<cword>")
			M.show_details(word)
		elseif command == "stats" then
			M.show_data()
		elseif command == "toggle" then
			M.toggle()
		else
			vim.notify("Invalid Vimscape command. Use: stats, details, toggle", vim.log.levels.WARN)
		end
	end, {
		nargs = "?",
		complete = function(arg_lead, cmd_line, cursor_pos)
			local commands = { "stats", "details", "toggle" }
			local matches = {}
			for _, cmd in ipairs(commands) do
				if cmd:find(arg_lead, 1, true) == 1 then
					table.insert(matches, cmd)
				end
			end
			return matches
		end,
		desc = "Vimscape plugin commands: stats, details, toggle"
	})
end

return M
