-- build.lua
-- Automatically discovered by lazy.nvim when no `build` field is specified.
-- Downloads a pre-built binary from GitHub Releases, or falls back to
-- building from source if the Rust toolchain is available.

local REPO = "jakewaldrip/vimscape2007.nvim"
local LIB_NAME = "vimscape_backend"

--- Resolve the plugin root directory (where this file lives).
local plugin_dir = vim.fn.fnamemodify(debug.getinfo(1, "S").source:sub(2), ":p:h")
local lua_dir = plugin_dir .. "/lua"
local backend_dir = plugin_dir .. "/vimscape_backend"
local target_path = lua_dir .. "/" .. LIB_NAME .. ".so"
local version_path = lua_dir .. "/." .. LIB_NAME .. "_version"

-- ---------------------------------------------------------------------------
-- Helpers
-- ---------------------------------------------------------------------------

---Yield a progress message to lazy.nvim's build UI.
---@param msg string
local function progress(msg)
	if coroutine.running() then
		coroutine.yield({ msg = "vimscape2007: " .. msg, level = vim.log.levels.TRACE })
	end
end

---Yield an info-level message to lazy.nvim's build UI.
---@param msg string
local function info(msg)
	if coroutine.running() then
		coroutine.yield({ msg = "vimscape2007: " .. msg, level = vim.log.levels.INFO })
	end
end

---Run a shell command synchronously and return success, stdout, stderr.
---@param cmd string
---@param cwd? string
---@return boolean success
---@return string stdout
---@return string stderr
local function run(cmd, cwd)
	local full_cmd = cmd
	if cwd then
		full_cmd = string.format("cd %s && %s", vim.fn.shellescape(cwd), cmd)
	end
	local stdout = vim.fn.system(full_cmd)
	local success = vim.v.shell_error == 0
	return success, vim.trim(stdout or ""), ""
end

---Read a file's contents, returning nil if it doesn't exist.
---@param path string
---@return string|nil
local function read_file(path)
	local f = io.open(path, "r")
	if not f then
		return nil
	end
	local content = f:read("*a")
	f:close()
	return vim.trim(content)
end

---Write a string to a file.
---@param path string
---@param content string
local function write_file(path, content)
	local f = io.open(path, "w")
	if not f then
		return
	end
	f:write(content)
	f:close()
end

-- ---------------------------------------------------------------------------
-- Platform detection
-- ---------------------------------------------------------------------------

---Map jit.os + jit.arch to a Rust target triple.
---@return string|nil triple
---@return string|nil extension  The cargo output extension (before renaming to .so)
local function get_target()
	local os_name = jit.os:lower()
	local arch = jit.arch:lower()

	if os_name == "osx" or os_name == "mac" then
		if arch == "arm64" then
			return "aarch64-apple-darwin", ".dylib"
		end
		return "x86_64-apple-darwin", ".dylib"
	end

	if os_name == "linux" then
		if arch == "arm64" then
			return "aarch64-unknown-linux-gnu", ".so"
		end
		return "x86_64-unknown-linux-gnu", ".so"
	end

	return nil, nil
end

-- ---------------------------------------------------------------------------
-- Version detection
-- ---------------------------------------------------------------------------

---Get the current plugin version from git tags.
---@return string|nil
local function get_version()
	local ok, tag = run("git describe --tags --exact-match HEAD 2>/dev/null", plugin_dir)
	if ok and tag ~= "" then
		return tag
	end
	-- Fall back to abbreviated commit SHA so local dev builds still get versioned
	local ok2, sha = run("git rev-parse --short HEAD 2>/dev/null", plugin_dir)
	if ok2 and sha ~= "" then
		return sha
	end
	return nil
end

---Check if the installed binary already matches the current version.
---@param version string
---@return boolean
local function is_up_to_date(version)
	local installed = read_file(version_path)
	if installed == nil then
		return false
	end
	return installed == version
end

-- ---------------------------------------------------------------------------
-- Download pre-built binary
-- ---------------------------------------------------------------------------

---@param version string  Git tag (e.g. "v0.1.0")
---@param triple string   Target triple
---@return boolean success
local function download_binary(version, triple)
	local url = string.format(
		"https://github.com/%s/releases/download/%s/%s.so",
		REPO,
		version,
		triple
	)
	local checksum_url = url .. ".sha256"
	local tmp_path = target_path .. ".tmp"

	progress("downloading binary for " .. triple .. "...")

	-- Download binary
	local ok = run(string.format(
		"curl --fail --location --silent --output %s %s",
		vim.fn.shellescape(tmp_path),
		vim.fn.shellescape(url)
	))
	if not ok then
		progress("download failed from " .. url)
		return false
	end

	-- Download and verify checksum
	progress("verifying checksum...")
	local checksum_ok, checksum_content = run(string.format(
		"curl --fail --location --silent %s",
		vim.fn.shellescape(checksum_url)
	))
	if checksum_ok and checksum_content ~= "" then
		-- The checksum file contains: "<hash>  <filename>"
		local expected_hash = checksum_content:match("^(%S+)")
		if expected_hash then
			local hash_ok, actual_hash = run(string.format(
				"sha256sum %s | cut -d' ' -f1",
				vim.fn.shellescape(tmp_path)
			))
			-- macOS uses shasum instead of sha256sum
			if not hash_ok then
				hash_ok, actual_hash = run(string.format(
					"shasum -a 256 %s | cut -d' ' -f1",
					vim.fn.shellescape(tmp_path)
				))
			end
			if hash_ok and vim.trim(actual_hash) ~= expected_hash then
				progress("checksum mismatch! Expected " .. expected_hash .. ", got " .. vim.trim(actual_hash))
				os.remove(tmp_path)
				return false
			end
		end
	else
		progress("checksum file not available, skipping verification")
	end

	-- Move into place (atomic rename to avoid partial reads)
	os.remove(target_path)
	local rename_ok, rename_err = os.rename(tmp_path, target_path)
	if not rename_ok then
		progress("failed to move binary into place: " .. (rename_err or "unknown error"))
		os.remove(tmp_path)
		return false
	end

	return true
end

-- ---------------------------------------------------------------------------
-- Build from source
-- ---------------------------------------------------------------------------

---@param cargo_ext string  The platform-specific lib extension cargo produces (e.g. ".dylib")
---@return boolean success
local function build_from_source(cargo_ext)
	-- Check if cargo is available
	local has_cargo = run("cargo --version")
	if not has_cargo then
		return false
	end

	progress("building from source (this may take a while)...")

	local ok, stdout = run("cargo build --release 2>&1", backend_dir)
	if not ok then
		progress("cargo build failed: " .. stdout)
		return false
	end

	-- Copy the compiled library to lua/
	local cargo_output = backend_dir .. "/target/release/lib" .. LIB_NAME .. cargo_ext
	local cp_ok, cp_err = run(string.format(
		"cp %s %s",
		vim.fn.shellescape(cargo_output),
		vim.fn.shellescape(target_path)
	))
	if not cp_ok then
		progress("failed to copy built library: " .. cp_err)
		return false
	end

	return true
end

-- ---------------------------------------------------------------------------
-- Main
-- ---------------------------------------------------------------------------

local function build()
	local triple, cargo_ext = get_target()
	if not triple or not cargo_ext then
		info("unsupported platform: " .. jit.os .. "/" .. jit.arch .. ". Build from source manually.")
		return
	end

	local version = get_version()
	if not version then
		progress("could not determine plugin version from git")
	end

	-- Skip if already up to date
	if version and is_up_to_date(version) then
		-- Also verify the binary actually exists
		local f = io.open(target_path, "r")
		if f then
			f:close()
			info("binary is up to date (" .. version .. ")")
			return
		end
	end

	progress("platform: " .. triple .. ", version: " .. (version or "unknown"))

	-- Attempt 1: Download pre-built binary (only if we have a proper tag version)
	local downloaded = false
	if version and version:match("^v") then
		downloaded = download_binary(version, triple)
	end

	-- Attempt 2: Build from source
	if not downloaded then
		if version and version:match("^v") then
			progress("pre-built binary not available, trying build from source...")
		else
			progress("not on a release tag, building from source...")
		end

		local built = build_from_source(cargo_ext)
		if not built then
			info(
				"could not download or build vimscape_backend. "
				.. "Install the Rust toolchain (https://rustup.rs) and run: "
				.. "cd vimscape_backend && cargo build --release"
			)
			return
		end
	end

	-- Record version so subsequent installs can skip
	if version then
		write_file(version_path, version)
	end

	info("build complete!")
end

build()
