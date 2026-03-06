local keys = require("keys")
local utils = require("utils")
local globals = require("globals")
local window_config = require("window_config")
local config = require("config")

local ok, vimscape = pcall(require, "vimscape_backend")
if not ok then
	vimscape = nil
end

local ns = vim.api.nvim_create_namespace("vimscape_keys")

local function get_db_full_path()
	return config.db_path .. config.db_name
end

local record_key = function(_, typed)
	keys.record_keys(typed, get_db_full_path(), config.batch_size, config)
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
	utils.notify("Vimscape2007 initialized", vim.log.levels.DEBUG)

	config = vim.tbl_deep_extend("force", config, opts or {})

	if not vimscape then
		vim.notify(
			"Vimscape2007: failed to load backend. Run ':Lazy build vimscape2007.nvim' or see README for build instructions.",
			vim.log.levels.ERROR
		)
		M.create_user_commands()
		return
	end

	vim.fn.mkdir(config.db_path, "p")

	local setup_ok, setup_err = pcall(vimscape.setup_tables, get_db_full_path())
	if not setup_ok then
		vim.notify(
			"Vimscape2007: database initialization failed: " .. tostring(setup_err),
			vim.log.levels.ERROR
		)
		M.create_user_commands()
		return
	end

	if config.token_log then
		vimscape.enable_token_log(get_db_full_path())
		utils.notify("Vimscape token logging enabled", vim.log.levels.DEBUG)
	end

	M.create_user_commands()
end

M.toggle = function()
	globals.set_active(not globals.get_active())

	if globals.get_active() then
		utils.notify("Vimscape recording started", vim.log.levels.INFO)
		vim.on_key(record_key, ns)
	else
		utils.notify("Vimscape recording paused", vim.log.levels.INFO)
		vim.on_key(nil, ns)
		vim.api.nvim_buf_clear_namespace(0, ns, 0, -1)
		globals.clear_typed_letters()
	end
end

M.show_data = function()
	if not vimscape then
		vim.notify("Vimscape2007: backend not loaded", vim.log.levels.ERROR)
		return
	end

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
	if not vimscape then
		vim.notify("Vimscape2007: backend not loaded", vim.log.levels.ERROR)
		return
	end

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

M.flush = function()
	if not vimscape then
		vim.notify("Vimscape2007: backend not loaded", vim.log.levels.ERROR)
		return
	end

	local typed_letters = globals.get_typed_letters()
	if #typed_letters == 0 then
		utils.notify("Vimscape: no keystrokes to flush", vim.log.levels.INFO)
		return
	end

	local count = #typed_letters
	local string_value = table.concat(typed_letters)
	vimscape.process_batch(string_value, get_db_full_path())
	globals.clear_typed_letters()
	utils.notify("Vimscape: flushed " .. count .. " keystrokes", vim.log.levels.INFO)
end

M.create_user_commands = function()
	vim.api.nvim_create_user_command("Vimscape", function(cmd_opts)
		local command = cmd_opts.args

		if command == "" then
			utils.notify("Vimscape commands: stats, details, toggle, flush", vim.log.levels.INFO)
			return
		end

		if command == "details" then
			local word = vim.fn.expand("<cword>")
			if word == "" then
				utils.notify("Place cursor on a skill name first", vim.log.levels.WARN)
				return
			end
			M.show_details(word)
		elseif command == "stats" then
			M.show_data()
		elseif command == "toggle" then
			M.toggle()
		elseif command == "flush" then
			M.flush()
		else
			utils.notify("Invalid Vimscape command. Use: stats, details, toggle, flush", vim.log.levels.WARN)
		end
	end, {
		nargs = "?",
		complete = function(arg_lead, _cmd_line, _cursor_pos)
			local commands = { "stats", "details", "toggle", "flush" }
			local matches = {}
			for _, cmd in ipairs(commands) do
				if cmd:find(arg_lead, 1, true) == 1 then
					table.insert(matches, cmd)
				end
			end
			return matches
		end,
		desc = "Vimscape plugin commands: stats, details, toggle, flush"
	})
end

return M
