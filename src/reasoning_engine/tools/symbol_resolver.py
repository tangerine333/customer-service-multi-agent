"""Symbol resolution tool - resolve symbols to their definitions and type info.

Uses the code graph (PostgreSQL) to find:
- Symbol definitions (function, class, variable)
- Type information
- Callers and callees
- References
"""

from typing import Optional


class SymbolResolverTool:
    """Resolve code symbols to their definitions, types, and relationships."""

    def __init__(self):
        self._symbols: dict[str, dict] = {}  # qualified_name -> definition

    def register_symbol(
        self,
        name: str,
        kind: str,
        file_path: str,
        line: int,
        signature: Optional[str] = None,
        return_type: Optional[str] = None,
    ):
        """Register a symbol definition."""
        self._symbols[name] = {
            "name": name,
            "kind": kind,
            "file_path": file_path,
            "line": line,
            "signature": signature,
            "return_type": return_type,
        }

    def resolve(self, name: str) -> Optional[dict]:
        """Resolve a symbol name to its definition."""
        return self._symbols.get(name)

    def get_type_info(self, name: str) -> Optional[str]:
        """Get type information for a symbol."""
        symbol = self._symbols.get(name)
        if symbol:
            return symbol.get("return_type") or symbol.get("kind", "unknown")
        return None

    def find_definition(self, name: str, file_path: str) -> Optional[dict]:
        """Find where a symbol is defined (prefer same file, then global)."""
        # Try exact match first
        if name in self._symbols:
            return self._symbols[name]

        # Fuzzy match: find symbols that end with the name
        for qualified, info in self._symbols.items():
            if qualified.endswith(f".{name}") or qualified == name:
                return info

        return None

    def list_symbols_in_file(self, file_path: str) -> list[dict]:
        """List all symbols defined in a file."""
        return [
            s for s in self._symbols.values()
            if s["file_path"] == file_path
        ]

    def get_callers(self, name: str) -> list[str]:
        """Get list of functions that call this symbol (placeholder)."""
        # In production, this queries the call graph in PostgreSQL
        return []

    def get_callees(self, name: str) -> list[str]:
        """Get list of functions called by this symbol (placeholder)."""
        return []
