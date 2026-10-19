#!/usr/bin/env python3
"""Print the tools advertised by a local Termos MCP server over stdio."""

import json
import select
import subprocess
import sys


def read_response(process: subprocess.Popen[str], request_id: int) -> dict:
    while True:
        ready, _, _ = select.select([process.stdout], [], [], 5)
        if not ready:
            raise TimeoutError("Timed out waiting for the Termos MCP server")
        line = process.stdout.readline()
        if not line:
            raise RuntimeError("Termos MCP server closed stdout unexpectedly")
        message = json.loads(line)
        if message.get("id") == request_id:
            return message


def main() -> int:
    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} /path/to/termos", file=sys.stderr)
        return 2

    process = subprocess.Popen(
        [sys.argv[1], "mcp"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
        bufsize=1,
    )
    try:
        assert process.stdin is not None
        process.stdin.write(
            json.dumps(
                {
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "initialize",
                    "params": {
                        "protocolVersion": "2025-03-26",
                        "capabilities": {},
                        "clientInfo": {"name": "termos-demo", "version": "1.0"},
                    },
                }
            )
            + "\n"
        )
        process.stdin.flush()
        read_response(process, 1)

        process.stdin.write(
            json.dumps(
                {"jsonrpc": "2.0", "method": "notifications/initialized"}
            )
            + "\n"
        )
        process.stdin.write(
            json.dumps(
                {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}
            )
            + "\n"
        )
        process.stdin.flush()

        response = read_response(process, 2)
        if "error" in response:
            raise RuntimeError(response["error"].get("message", "tools/list failed"))

        tools = response.get("result", {}).get("tools", [])
        print("Default Termos MCP tools:")
        for tool in tools:
            print(f"  {tool['name']}")
        return 0
    finally:
        if process.stdin is not None:
            process.stdin.close()
        process.terminate()
        try:
            process.wait(timeout=2)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, RuntimeError, TimeoutError, json.JSONDecodeError) as error:
        print(f"MCP demo failed: {error}", file=sys.stderr)
        raise SystemExit(1)
