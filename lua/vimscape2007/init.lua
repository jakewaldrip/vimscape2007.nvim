local keys = require("keys")
local utils = require("utils")
local globals = require("globals")
local window_config = require("window_config")
local vimscape = require("vimscape_backend")

local M = {}
local ns = vim.api.nvim_create_namespace("vimscape_keys")

M.setup = function(opts)
	vim.notify("Ran setup for Vimscape", vim.log.levels.INFO)

	-- TODO: Do this via deep_copy table if possible, storing the config in
	-- a separate config module instead of in globals
	globals.db_path = opts.db_path
	vimscape.setup_tables(globals.db_path)
end

M.toggle = function()
	globals.active = not globals.active

	if globals.active then
		print("Recording active")
	else
		print("Recording stopped")
		vim.api.nvim_buf_clear_namespace(0, ns, 0, -1)
		globals.typed_letters = {}
	end
end

M.show_data = function()
	-- Restore modifiable after closing the window
	-- TODO ensure this is reliable, possibly remove the autocommand at the end
	vim.api.nvim_create_autocmd({ "BufLeave" }, {
		callback = function()
			vim.api.nvim_set_option_value("modifiable", true, {})
		end,
	})

	vim.keymap.set("n", "q", ":q<CR>", { silent = true, buffer = window_config.vimscape_stats_bufnr })
	vim.keymap.set(
		"n",
		"d",
		":lua print('Getting Details')<CR>",
		{ silent = true, buffer = window_config.vimscape_stats_bufnr }
	)

	local bufr_width = window_config.stat_window_config.width
	local user_data = vimscape.get_user_data(bufr_width, globals.db_path)
	print(utils.dump(user_data))
	vim.api.nvim_open_win(window_config.vimscape_stats_bufnr, true, window_config.stat_window_config)

	vim.api.nvim_buf_set_lines(window_config.vimscape_stats_bufnr, 0, -1, false, {})

	-- vim.api.nvim_set_option_value("modifiable", true, {})
	for k, v in pairs(user_data) do
		local text = {}
		text[1] = v
		vim.api.nvim_buf_set_lines(window_config.vimscape_stats_bufnr, k, k, false, text)
	end
	-- vim.api.nvim_set_option_value("modifiable", false, {})
end

vim.on_key(keys.record_keys, ns)

return M
