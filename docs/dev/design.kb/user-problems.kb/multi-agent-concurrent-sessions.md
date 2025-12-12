# Multi-Agent Concurrent Sessions

Multiple Claude Code agents work simultaneously in the same repository.

## Context

User's dotfiles repo at `~` has multiple agents making changes concurrently. Each agent needs to commit its own changes without interfering with others.

## Requirements

1. Each agent gets isolated partial-commit state
2. No collision between concurrent sessions
3. No explicit coordination required between agents
4. Sessions are independent - one agent's commit doesn't affect another's hunks
