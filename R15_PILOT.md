# R15 usability pilot

Status: **not executed**. Five engineering participants are required to close
NEXT_STEPS.md Phase 2.5. Automated browser journeys verify functionality, but
do not establish human task completion times or discoverability.

## Session setup

Use the same production build for all five sessions. Record its commit, URL,
browser/version, device, participant ID (P1–P5), and prior modeling experience.
Start each session with a fresh project and default settings. Explain that
participants may use the built-in help and should think aloud. Do not show
the facilitator answers below. Record elapsed time from revealing each task
to a correct solved result; record every hint and error separately. Stop the
timer at the task limit and offer assistance afterward, marking the attempt
as assisted/incomplete. Do not count an assisted completion as an unaided pass.

## Participant task cards

1. **Scalar model — 5 minutes.** Model a 12 V supply driving a 30 Ω resistance.
   Calculate current and electrical power, and show the solved values with
   units. Start from a blank document.
2. **Component chain — 10 minutes.** Build and solve a closed circuit using a
   12 V VoltageSource, 10 Ω and 20 Ω Resistors in series, and Ground. Use the
   component help to find the connections. Identify the series current.
3. **Missing boundary recovery — 3 minutes.** Open the document below. The
   intended current is 0.4 A. Diagnose why it cannot yet be solved, supply the
   missing physical condition, and obtain resistance and power.

   ```frees
   V = 12 [V]
   V = I * R
   P = V * I
   ```

After each task, ask: “What was hardest to find or understand?” and “How
confident are you in the result, from 1 to 5?” Record the participant's words
without inferring success from confidence alone.

## Facilitator answer key

Task 1: current 0.4 A and power 4.8 W. One valid document is:

```frees
V = 12 [V]
R = 30 [ohm]
V = I * R
P = V * I
```

Task 2: series current magnitude 0.4 A. This is the circuit used by the
existing browser component journey:

```frees
VoltageSource V1(E = 12)
Resistor R1(R = 10)
Resistor R2(R = 20)
Ground G1()
connect(V1.p, R1.a)
connect(R1.b, R2.a)
connect(R2.b, V1.n, G1.port)
```

Task 3: the check reports an underspecified model; adding `I = 0.4 [A]`
gives R = 30 Ω and P = 4.8 W. Accept equivalent independent conditions that
express the stated current. Do not accept merely deleting an equation or
silencing a diagnostic.

## Results and acceptance

Build commit: pending. Session dates: pending. Facilitator: pending.

| Participant | Experience / browser / device | Scalar seconds / outcome | Chain seconds / outcome | Recovery seconds / outcome | Hints, confidence and observations |
| --- | --- | --- | --- | --- | --- |
| P1 | pending | pending | pending | pending | pending |
| P2 | pending | pending | pending | pending | pending |
| P3 | pending | pending | pending | pending | pending |
| P4 | pending | pending | pending | pending | pending |
| P5 | pending | pending | pending | pending | pending |

Use outcomes `unaided`, `assisted`, or `incomplete`, recording actual seconds
even when a limit is missed. Report each task's unaided completion count out
of five, median time among unaided completions, and all missed time limits.
Do not replace incomplete attempts with zero or hide them in the median.

The roadmap's targets are ≤300 / ≤600 / ≤180 seconds respectively. Keep 2.5
open until five sessions are recorded and findings are triaged. Treat all five
participants meeting each target unaided as a pass; otherwise record the
specific failures, make the relevant fixes and retest before claiming the
targets are met. Any decision to accept a missed target must be recorded by
the project owner. Track each finding with task, observed behavior, impact,
issue/fix reference, and retest result.
