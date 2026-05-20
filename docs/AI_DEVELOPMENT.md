# AI-Assisted Development Notes

This document exists for transparency: **JerichoOS was built with meaningful AI assistance**, primarily **Claude Sonnet 4.5** (Sep 29, 2025). It was occasionally used when I needed deeper analysis.  

I’m writing this for future maintainers and any curious readers: not to sell AI, and not to demonize it; just to describe what actually happened.

> **AI isn’t inherently good or bad...**  
> It’s a tool that amplifies intent, habits, and discipline. In systems work, that amplification can be beautiful or dangerous.

---

## Why Document This?

Bare-metal OS development is an unforgiving place. Code can look perfect and still be wrong in ways that only show up at boot, under interrupts, or under real timing and environment constraints.

So the philosophy was essentially just: 
1. **Trust output less than you trust tests**
2. And treat AI as an accelerator.

---

## Ethical Note (my stance)

I don’t view AI assistance as cheating or as virtue, it’s literally just a tool with consequences.

- I **do** support using AI to learn, to prototype, to document, and to explore designs faster *when you remain accountable for the result*.
- I **don’t** condone using AI to misrepresent authorship, bypass learning you’re claiming you did, or automate harm (emphasis on this last part).
- In general, if the work is safety-critical, security-sensitive, or affects other people’s trust, the bar for verification and honesty should be higher.

This repo tries to meet that bar by being explicit about what AI was used for and what humans verified.

---

## What AI Was Actually Good At

### Design + tradeoffs
AI was strongest as a thinking partner, for the following:
- proposing architectures and alternatives
- comparing tradeoffs
- pointing out missing invariants or unclear boundaries

**Best use:** discussions/decomposition

### Debugging hypotheses
AI helped a lot when I fed it real context:
- error logs, command lines, diffs, disassembly snippets, etc
- symptoms + what I already tried

It was great at quickly generating a shortlist of plausible root causes.

**Reality check:** it can still be confidently wrong; incomplete context.

### Docs + communication
AI did well in turning messy technical notes into readable documentation:
- READMEs, setup steps, high-level explanations, design notes.

**Reality check:** docs still need review.. AI can fill in details that sound right but aren't.

### Boilerplate
AI saved significant time on repetitive work:
- build scripts, CI scaffolding, config wiring stuff
- templates and skeletons that are easy to validate

---

## Where AI Was Weak (and needed STRICT supervision)

### Hardware/boot details 
ARM64 boot and exception-level transitions are subtle. AI output often looked legit while still containing very critical mistakes.

### Stateful logic + concurrency 
AI can produce code that compiles and *feels* right but violates invariants. Invariants are extremely useful. Do stress tests and assert aggressively.

### `unsafe` Rust 
AI can be casual with aliasing, alignment, lifetimes, volatile/MMIO semantics, and pointer math.

### Environment-specific behavior
WSL2/QEMU, buffering issues, toolchain differences. As obvious, AI cannot predict these.

---

## Workflow That Worked

A tight loop kept AI helpful:

1. **Human** defines constraints + acceptance tests  
2. **AI** drafts an implementation or suggests options  
3. **Human** reviews like a code reviewer  
4. **Human** runs tests / reproduces issues  
5. **AI** helps debug and propose *minimal* fixes  
6. Repeat

**Non-negotiable:** every meaningful AI change was reviewed, tested, and committed in small steps.

---

## Closing Note

AI isn't dangerous. I'm not a purist. What I refuse is dependence on something I don't fully understand.

So I practice deliberately without it:
- System design exercises with no autocomplete or assistant.
- Sit in the discomfort of not knowing and reason through it.
- Build small projects in full isolation.

The goal is simple: if the tools disappeared tomorrow, I'd still be capable.
