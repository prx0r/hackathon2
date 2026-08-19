# Hackathon Master Strategy — Evidence-Based Winning Framework

*Compiled from TAIKAI, Devpost, ETHGlobal, JetBrains, NASA, Google GKE, DoraHacks, AngelHack, academic research, and repeat winners*

---

## The central finding

For a solo remote developer, the hackathon is two products:

**Product A** — the thing you built.
**Product B** — the judge-comprehension interface around it.

Product B can determine whether Product A ever receives serious scrutiny. Devpost judges check requirements and watch the video before deeper evaluation; some competitions judge solely from description/images/video.

## The hierarchy to optimize

```
Eligibility → Rubric alignment → Judge comprehension →
Memorable demo → Working proof → Technical depth → Breadth
```

A 95/100 idea with one disqualifying requirement = **0**.

---

## Stage 0: Hackathon Intelligence File

Before generating any ideas, turn the event into a structured specification:

```yaml
hackathon:
  objective:
  challenge_statement:
  judging:
    criteria:
      - name:
        weight:
        evidence_required:
    async_first_round: true/false
    live_final: true/false
  constraints:
    solo_allowed:
    existing_code_allowed:
    sponsor_technology_required:
    ai_usage_allowed:
  submission:
    video_required:
    video_max_seconds:
    repo_required:
    live_demo:
  judges:
    - name:
      role:
      technicality:
      interests:
```

## Hard gate: ELIGIBILITY

Every idea gets pass/fail:

| Gate | Requirement |
|------|-------------|
| Challenge fit | Directly answers the actual prompt |
| Rules | No prohibited pre-existing work |
| Solo feasibility | One developer can deliver demo-critical path |
| Required tech | Sponsor/API genuinely used |
| Demonstrability | Core claim can be shown visibly |
| Submission feasibility | Video/deployment/docs achievable |
| Verifiability | Judge can tell it genuinely works |
| Attribution | Existing code/data/AI appropriately disclosed |
| Access | Judge can open every required artifact |
| Time | Core demo can be finished with buffer |

**Any hard failure → reject idea.**

---

## Universal idea rubric (100 points)

| Criterion | Weight | Agent question |
|-----------|-------:|----------------|
| Rubric fit | 15 | Does this maximize the actual published criteria? |
| Problem strength | 10 | Is there an identifiable painful/valuable problem? |
| Original insight | 12 | What is the non-obvious thing here? |
| **Demoability** | **16** | Can its value become obvious visually in ≤90 sec? |
| Technical depth | 14 | Is there real engineering beyond a generic wrapper? |
| Functional completeness | 10 | Can one user journey work end-to-end? |
| Impact / utility | 7 | Would someone plausibly want this? |
| Sponsor-native leverage | 6 | Does required tech enable the magic? |
| UX / judgeability | 5 | Can a stranger use/understand it instantly? |
| Memorability / wow | 5 | What one thing will judge remember after 50 entries? |

Then apply the actual event rubric:

```
WIN_SCORE =
    0.70 × EVENT_RUBRIC_SCORE
  + 0.30 × UNIVERSAL_SCORE
```

If competition gives Presentation 40%, do not let generic rubric override it.

---

## Demo Compression Ratio (DCR)

```
DCR = perceived_value / seconds_required_to_demonstrate_value
```

High DCR: Upload ugly data → click button → impossible-looking useful output appears.

Low DCR: First explain distributed trust, then architecture, then create account, configure four services, wait for indexing...

Target:
```yaml
demo_compression:
  seconds_until_problem_understood: <15
  seconds_until_product_visible: <30
  seconds_until_magic_moment: <90
  setup_actions_visible: <=2
  primary_user_flow_count: 1
```

---

## Reject the 2026 "AI wrapper penalty"

Run WRAPPER_TEST:

```
value proposition ≈
    chat interface
  + standard LLM API
  + ordinary system prompt
```

Require at least one hard differentiator:
- new workflow
- unusual interaction
- proprietary/novel dataset
- meaningful external-world action
- verification mechanism
- novel architecture
- multi-step orchestration with measurable benefit
- benchmarked performance improvement

---

## One impressive completed loop beats five unfinished capabilities

Feature classification:

```yaml
feature:
  required_for_magic_demo: true/false
  required_by_rules: true/false
  directly_increases_rubric_score: true/false
```

If all three are false: **CUT IT.**

Common sinkholes to cut:
- authentication
- account settings
- elaborate permissions
- generic dashboards
- extensive logging UIs
- complete billing
- production-scale infra
- numerous modes
- five integrations when one suffices

---

## Build sequence for solo remote dev

```
1. Define winning demo sentence
2. Storyboard 60-120 second user journey
3. Implement the critical path
4. Make critical path deterministic
5. Add visible proof that the hard thing happened
6. Add technical depth that improves rubric score
7. Polish only screens appearing in demo
8. Build architecture diagram
9. Record benchmark/evidence
10. Write submission
11. Record final video
12. Test every public link logged-out/incognito
```

---

## The 3-minute video template

### 0:00-0:10 — Hook
One sentence: "[User] currently suffers [specific pain]. [Product] turns [before] into [after]."
No logo animation.

### 0:10-0:25 — Evidence/problem
One real number, screenshot, workflow or concrete example.

### 0:25-0:35 — Product thesis
"So I built X." State the differentiator.

### 0:35-1:45 — THE DEMO
One uninterrupted flow: Input → action → magic → result.

### 1:45-2:15 — Why technically difficult
Architecture graphic. 2-4 genuinely interesting engineering decisions.

### 2:15-2:35 — Proof
Latency, benchmark, accuracy, tests, cost reduction, verification.

### 2:35-2:50 — Why sponsor tech matters
Not "I used Foo API." Instead: "Foo's capability X makes Y possible; without it we'd need Z."

### 2:50-3:00 — Memorable closure
Return to initial problem. One sentence. Done.

---

## Zero-Click Comprehension Test

From the submission page alone, within ~30 seconds, a judge should know:

```
WHO has the problem?
WHAT is the problem?
WHAT did you build?
WHAT does it visibly do?
WHAT is new?
WHAT difficult engineering did you do?
HOW does the required technology matter?
WHERE is the demo?
WHERE is the code?
```

---

## README optimized for judges

```markdown
# PRODUCT NAME
> One-line result-oriented description.

[DEMO GIF / IMAGE]

## The Problem
3-5 sentences + evidence.

## What It Does
Concrete description.

## Why It's Different
3 bullets maximum.

## Demo
Video + Live deployment

## Architecture
Diagram (5-9 boxes, not a cloud poster)

## How It Works
1. 2. 3. 4.

## Technical Highlights
Actual interesting engineering.

## Sponsor / Required Technology
Exactly where it is used and why.

## Results
Benchmarks / tests / metrics.

## Reproduce
Minimal setup commands.

## What Was Built During the Hackathon
Explicitly separate new vs reused.

## Limitations
Concise and credible.
```

---

## Judge simulation (run before submission)

### Judge A — tired generalist
Give: title, first paragraph, first 45 sec video.
Ask: What does this do? Who is it for? Why different? What do you remember?

### Judge B — technical
Give: repo, architecture, technical section.
Ask: What is actually difficult? What appears reused? What evidence shows it works?

### Judge C — sponsor
Ask: Would this product still be basically identical without our technology?

### Judge D — hostile verifier
Ask: Find every statement not supported by a demo, benchmark, code location or external evidence.

### Judge E — rubric scorer
Give only official criteria. Require evidence for each score.

---

## Red flags (penalize brutally)

| Red flag | Penalty |
|----------|--------:|
| Doesn't clearly answer challenge | REJECT |
| Required integration superficial | -20 |
| Demo isn't actually functional | -20 |
| Generic LLM wrapper | -15 |
| No clear user/problem | -15 |
| Requires >30 sec explanation before product appears | -10 |
| Five mediocre features instead of one killer flow | -10 |
| Judge must install complicated local stack | -10 |
| Architecture sounds impressive but isn't demonstrated | -8 |
| Marketing claims without evidence | -8 |
| README hides required integration | -8 |
| Broken/private link | potentially REJECT |

## Positive multipliers

```
+ problem personally experienced
+ strong external evidence
+ visibly transformative interaction
+ surprising but immediately understandable
+ real technical challenge
+ meaningful sponsor integration
+ usable public deployment
+ independently reproducible
+ benchmarked improvement
+ clear before/after
+ single strong end-to-end flow
+ existing alternatives clearly inferior
```

---

## Final scoring

```
ELIGIBILITY = all(hard_gates)

EVENT = normalized official rubric score
UNIVERSAL = universal score above

COMPREHENSION =
    title_clarity + problem_clarity + differentiation_clarity + demo_clarity

EVIDENCE =
    functional_proof + technical_proof + impact_proof + sponsor_proof

SUBMISSION_RELIABILITY =
    public_links + deployment + video + repo + reproduction

FINAL =
    if !ELIGIBILITY: 0
    else:
      0.55 * EVENT
    + 0.15 * UNIVERSAL
    + 0.12 * COMPREHENSION
    + 0.10 * EVIDENCE
    + 0.08 * SUBMISSION_RELIABILITY
```

### Release thresholds

```
< 70  → do not submit
70-79 → viable
80-87 → competitive
88-93 → finalist-quality
94+   → exceptionally strong
```

---

## Memory hook

Every idea must be expressible as:

> "Oh, that's the project that ______."

If the blank takes three sentences, penalize.

---

## The meta-rule

> **Build backward from the moment of judgment.**

Choose a real problem → identify one surprising solution → construct one technically credible end-to-end mechanism → make the mechanism visibly work → ensure every judging criterion is evidenced → eliminate everything that doesn't improve those things → package it so an exhausted stranger understands the achievement immediately.

**Treat judgeability itself as an engineering constraint.**
