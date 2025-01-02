local vimscape = require("vimscape2007")
local keys = require("keys")
local utils = require("utils")
local globals = require("globals")
local window_config = require("window_config")

local M = {}

local ns = vim.api.nvim_create_namespace("vimscape_keys")

M.setup = function(opts)
	print("Options: ", opts)
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

	local user_data = vimscape.get_user_data("")
	print(utils.dump(user_data))
	vim.api.nvim_open_win(window_config.vimscape_stats_bufnr, true, window_config.stat_window_config)

	vim.api.nvim_set_option_value("modifiable", true, {})
	for k, v in pairs(user_data) do
		print(k, v.skill_name, v.level, v.total_exp)

		-- Figure out why this isn't triggering
		-- Is it possibly conflicting with another keymap?
		-- vim.api.nvim_buf_set_keymap(Vimscape_stats_bufnr, "n", "q", ":lua print('hello')", {})
		-- Create one for viewing the details as well (another smaller window, do this later its just for vibe)
		-- vim.api.nvim_buf_set_keymap(Vimscape_stats_bufnr, "n", "d", ":lua print('hello')", {})

		local text = {}
		local line = "Skill: " .. v.skill_name .. " | Level: " .. v.level .. " | Total Exp: " .. v.total_exp
		text[1] = line
		vim.api.nvim_buf_set_lines(window_config.vimscape_stats_bufnr, k, k, false, text)
	end
	vim.api.nvim_set_option_value("modifiable", false, {})
end

vim.on_key(keys.record_keys, ns)

return M
