local globals = require("globals")
local utils = require("utils")

local ok, vimscape = pcall(require, "vimscape_backend")
if not ok then
	vimscape = nil
end

---@class Keys
local M = {}

---@return string | nil
M.sanitize_key = function(key)
	local b = key:byte()
	if b and b >= 33 and b <= 126 then
		return key
	end

	local translated = vim.fn.keytrans(key)

	local skip_patterns = {
		"^<t_..>$",  -- Terminal codes
		"<Cmd>",    -- Command key
		"<LeftMouse>", "<RightMouse>", "<MiddleMouse>", "<Mouse>", "<ScrollWheel>", "<LeftDrag>", "<LeftRelease>", "<RightDrag>", "<RightRelease>", "<MiddleDrag>", "<MiddleRelease>", "<2-LeftMouse>", "<2-RightMouse>", "<2-MiddleMouse>", "<3-LeftMouse>", "<3-RightMouse>", "<3-MiddleMouse>", "<4-LeftMouse>", "<4-RightMouse>", "<4-MiddleMouse>",
		"<F13>", "<F14>", "<F15>", "<F16>", "<F17>", "<F18>", "<F19>", "<F20>", "<F21>", "<F22>", "<F23>", "<F24>", "<F25>", "<F26>", "<F27>", "<F28>", "<F29>", "<F30>", "<F31>", "<F32>", "<F33>", "<F34>", "<F35>", "<F36>", "<F37>",
	}

	for _, pattern in ipairs(skip_patterns) do
		if translated:match(pattern) then
			return nil
		end
	end

	local translate_map = {
		["<CR>"] = "|enter|",
		["<Tab>"] = "|tab|",
		["<BS>"] = "|backspace|",
		["<Del>"] = "|delete|",
		["<Space>"] = "|space|",
		["<Esc>"] = "|escape|",
		["<Up>"] = "|up|",
		["<Down>"] = "|down|",
		["<Left>"] = "|left|",
		["<Right>"] = "|right|",
		["<Home>"] = "|home|",
		["<End>"] = "|end|",
		["<PageUp>"] = "|pageup|",
		["<PageDown>"] = "|pagedown|",
		["<Insert>"] = "|insert|",
	}

	if translate_map[translated] then
		return translate_map[translated]
	end

	return translated
end

M.record_keys = function(typed, db_path, batch_size, config)
	if not globals.get_active() then
		return
	end

	-- Skip internally-generated key events (decomposed sub-operations).
	-- vim.on_key's second arg (typed) is empty for keys produced by
	-- Neovim's internal expansion of a previously typed key (e.g. <C-U>
	-- decomposes into many <C-E> events, each with typed="").
	if typed == nil or typed == "" then
		return
	end

	local mode = vim.api.nvim_get_mode().mode
	if mode == "i" then
		return
	end

	local new_key = M.sanitize_key(typed)

	if new_key == nil then
		return
	end

	if config.key_overrides[new_key] then
		new_key = config.key_overrides[new_key]
	end

	local typed_letters = globals.get_typed_letters()
	if #typed_letters >= batch_size and vimscape then
		local string_value = table.concat(typed_letters)
		vimscape.process_batch(string_value, db_path)
		utils.notify("Processed batch", vim.log.levels.DEBUG)
		globals.clear_typed_letters()
	end

	table.insert(typed_letters, new_key)
end

return M
