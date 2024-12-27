local vimscape = require("vimscape2007")

local typed_letters = {}
local ns = vim.api.nvim_create_namespace("vimscape_keys")
local active = false

local M = {}

M.setup = function(opts)
	print("Options: ", opts)
end

local sanitize_key = function(key)
	local b = key:byte()
	if b <= 126 and b >= 33 then
		return key
	end

	local translated = vim.fn.keytrans(key)

	-- Mouse events
	if translated:match("Left") or translated:match("Mouse") or translated:match("Scroll") then
		return nil
	end

	-- Keybound events show up as this for some reason? Skip 'em
	if translated:match("^<t_..>$") then
		return nil
	end

	-- Ignore Escape
	if translated:match("<Cmd>") then
		return nil
	end

	if translated:match("<CR>") then
		return "|enter|"
	end

	return translated
end

local record_keys = function(key)
	-- Return if we're not actively listening
	if not active then
		return
	end

	-- Ignore insert mode
	local mode = vim.api.nvim_get_mode().mode
	if mode == "i" then
		return
	end

	local new_key = sanitize_key(key)

	-- print("Key: ", new_key)

	if new_key == nil then
		return
	end

	if #typed_letters >= 50 then
		local string_value = table.concat(typed_letters)
		vimscape.process_batch(string_value)
		typed_letters = {}
	end

	table.insert(typed_letters, new_key)
end

M.toggle = function()
	active = not active

	if active then
		print("Recording active")
	else
		print("Recording stopped")
		vim.api.nvim_buf_clear_namespace(0, ns, 0, -1)
		typed_letters = {}
	end
end

local function dump(o)
	if type(o) == "table" then
		local s = "{ "
		for k, v in pairs(o) do
			if type(k) ~= "number" then
				k = '"' .. k .. '"'
			end
			s = s .. "[" .. k .. "] = " .. dump(v) .. ","
		end
		return s .. "} "
	else
		return tostring(o)
	end
end

local function round(float)
	return math.floor(float + 0.5)
end

Vimscape_stats_bufnr = vim.api.nvim_create_buf(false, true)
local ui = vim.api.nvim_list_uis()[1]
local width = round(ui.width)
local height = round(ui.height)

local footer_text = "[q]uit | [d]etails"

local stat_window_config = {
	relative = "editor",
	width = round(width / 2),
	height = round(height / 2),
	col = round(width / 4),
	row = round(height / 4),
	style = "minimal",
	focusable = false,
	border = "double",
	title = "Vimscape Skills",
	footer = footer_text,
}

M.show_data = function()
	-- Restore modifiable after closing the window
	vim.api.nvim_create_autocmd({ "BufLeave" }, {
		callback = function()
			vim.api.nvim_set_option_value("modifiable", true, {})
		end,
	})

	local user_data = vimscape.get_user_data("")
	vim.api.nvim_open_win(Vimscape_stats_bufnr, true, stat_window_config)

	vim.api.nvim_set_option_value("modifiable", true, {})
	for k, v in pairs(user_data) do
		print(k, v.skill_name, v.level, v.total_exp)
		-- Figure out why this isn't triggering
		-- vim.api.nvim_buf_set_keymap(Vimscape_stats_bufnr, "n", "q", ":lua print('hello')", {})
		-- Create one for viewing the details as well (another smaller window, do this later its just for vibe)
		-- vim.api.nvim_buf_set_keymap(Vimscape_stats_bufnr, "n", "d", ":lua print('hello')", {})

		local text = {}
		local line = "Skill: " .. v.skill_name .. " | Level: " .. v.level .. " | Total Exp: " .. v.total_exp
		text[1] = line
		vim.api.nvim_buf_set_lines(Vimscape_stats_bufnr, k, k, false, text)
	end
	vim.api.nvim_set_option_value("modifiable", false, {})
end

vim.on_key(record_keys, ns)

return M
