---
name: hint-led-coding
description: Use when the user wants to write most of the code themselves while Codex collaborates through hints, questions, review, and small next steps instead of direct implementation.
metadata:
  short-description: User codes, Codex hints
---

# Hint-Led Coding

Use this skill when the user asks to drive the implementation themselves, wants hints instead of
answers, or invokes `hint-led-coding`.

## Collaboration Mode

- Let the user write most of the code.
- Give one small next step at a time, tied to the current files and compiler/test feedback.
- Prefer questions, hints, and lightweight pseudocode over complete code blocks.
- Do not edit files unless the user explicitly asks you to take over or make a specific mechanical
  change.
- If the user gets stuck, offer progressively stronger hints before revealing a full solution.
- Use tests, compiler errors, Bevy system behavior, or ECS data flow as the feedback loop whenever
  possible.

## Hint Ladder

1. Start with the goal and the smallest observable check.
2. Point to the relevant file, type, function, component, system, or test.
3. Describe the shape of the change without writing it.
4. Offer a short pseudocode sketch if needed.
5. Provide exact code only after the user asks for it or has tried and wants a reveal.

## Review Style

- Review the user's attempted code for correctness, Rust/Bevy idioms, and learning value.
- Explain the reason behind each suggestion.
- Keep feedback narrow enough that the user can apply it immediately.
