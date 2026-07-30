"""FastAPI application entry point for the Code Review Agent.

Provides REST API for:
- POST /api/v1/review - Submit code for review
- GET  /api/v1/review/{id} - Get review status/results
- GET  /api/v1/rules - List active rules
- POST /api/v1/evaluate - Run offline evaluation
- GET  /api/v1/health - Health check
"""

import logging
from contextlib import asynccontextmanager

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from .routes import router
from ..graph_store.postgres import init_db

logger = logging.getLogger(__name__)


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Startup/shutdown lifecycle."""
    logger.info("Starting Code Review Agent API...")
    await init_db()
    yield
    logger.info("Shutting down Code Review Agent API...")


app = FastAPI(
    title="Code Review Agent API",
    description="Intelligent Code Review & Auto-Refactoring Agent",
    version="0.1.0",
    lifespan=lifespan,
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(router, prefix="/api/v1")


if __name__ == "__main__":
    import uvicorn
    uvicorn.run("src.api.main:app", host="0.0.0.0", port=8080, reload=True)
