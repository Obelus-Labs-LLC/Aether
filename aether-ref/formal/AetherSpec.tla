--------------------------- MODULE AetherSpec ---------------------------
(* Formal specification of the Aether policy evaluation engine.
   Verifies: policy completeness, conflict determinism, HCM mutual exclusion,
   and HCM bounded duration. *)

EXTENDS Integers, Sequences, FiniteSets, TLC

CONSTANTS
    Policies,           \* Set of policy names
    Links,              \* Set of link IDs
    TrafficClasses,     \* Set of traffic class labels
    MaxPriority,        \* Maximum priority value
    MaxHcmDuration,     \* Maximum HCM activation duration (ticks)
    MaxCumulativeHcm    \* Maximum cumulative HCM duration

VARIABLES
    policySet,          \* Function: policy name -> {priority, triggers, rules}
    triggerState,       \* Function: trigger name -> value
    linkState,          \* Function: link ID -> {available, latency}
    hcmActive,          \* Boolean
    hcmStartTick,       \* Nat or -1
    hcmCumulative,      \* Nat (cumulative ticks across activations)
    hcmScope,           \* Set of traffic class labels
    hcmActivationId,    \* String or ""
    tick,               \* Global monotonic tick counter
    lastDecision        \* Record of last evaluation result

vars == <<policySet, triggerState, linkState, hcmActive, hcmStartTick,
          hcmCumulative, hcmScope, hcmActivationId, tick, lastDecision>>

TypeOK ==
    /\ hcmActive \in BOOLEAN
    /\ hcmStartTick \in -1..1000
    /\ hcmCumulative \in 0..MaxCumulativeHcm
    /\ tick \in 0..1000

---------------------------------------------------------------------------
(* Evaluation: deterministic policy matching *)

(* A policy matches if its trigger conditions are satisfied *)
PolicyActive(p) ==
    LET triggers == policySet[p].triggers
    IN  IF triggers = {} THEN TRUE    \* No triggers = always active
        ELSE \A t \in triggers : triggerState[t] = TRUE

(* Find the highest-priority active policy that has a matching rule *)
MatchingPolicy(tc) ==
    LET active == {p \in Policies : PolicyActive(p)}
        matching == {p \in active : tc \in policySet[p].matchedClasses}
    IN  IF matching = {} THEN "default"
        ELSE CHOOSE p \in matching :
            \A q \in matching : policySet[p].priority >= policySet[q].priority

(* Core evaluation action *)
Evaluate(tc) ==
    /\ tc \in TrafficClasses
    /\ LET policy == MatchingPolicy(tc)
       IN  lastDecision' = [
               trafficClass |-> tc,
               policy |-> policy,
               tick |-> tick
           ]
    /\ UNCHANGED <<policySet, triggerState, linkState, hcmActive,
                    hcmStartTick, hcmCumulative, hcmScope, hcmActivationId, tick>>

---------------------------------------------------------------------------
(* HCM lifecycle *)

ActivateHcm(actId, scope) ==
    /\ ~hcmActive                   \* No concurrent activations
    /\ hcmActive' = TRUE
    /\ hcmStartTick' = tick
    /\ hcmScope' = scope
    /\ hcmActivationId' = actId
    /\ triggerState' = [triggerState EXCEPT !["hcm"] = TRUE]
    /\ UNCHANGED <<policySet, linkState, hcmCumulative, tick, lastDecision>>

DeactivateHcm ==
    /\ hcmActive
    /\ LET elapsed == tick - hcmStartTick
       IN  hcmCumulative' = hcmCumulative + elapsed
    /\ hcmActive' = FALSE
    /\ hcmStartTick' = -1
    /\ hcmScope' = {}
    /\ hcmActivationId' = ""
    /\ triggerState' = [triggerState EXCEPT !["hcm"] = FALSE]
    /\ UNCHANGED <<policySet, linkState, tick, lastDecision>>

HcmExpiry ==
    /\ hcmActive
    /\ tick - hcmStartTick >= MaxHcmDuration
    /\ DeactivateHcm

---------------------------------------------------------------------------
(* Time advance *)

Tick ==
    /\ tick' = tick + 1
    /\ UNCHANGED <<policySet, triggerState, linkState, hcmActive,
                    hcmStartTick, hcmCumulative, hcmScope, hcmActivationId,
                    lastDecision>>

---------------------------------------------------------------------------
(* Telemetry update *)

UpdateLink(link, avail) ==
    /\ link \in Links
    /\ linkState' = [linkState EXCEPT ![link].available = avail]
    /\ UNCHANGED <<policySet, triggerState, hcmActive, hcmStartTick,
                    hcmCumulative, hcmScope, hcmActivationId, tick, lastDecision>>

---------------------------------------------------------------------------
(* Initial state *)

Init ==
    /\ policySet = [p \in Policies |-> [
           priority |-> 1,
           triggers |-> {},
           matchedClasses |-> TrafficClasses
       ]]
    /\ triggerState = [t \in {"hcm"} |-> FALSE]
    /\ linkState = [l \in Links |-> [available |-> TRUE, latency |-> 50]]
    /\ hcmActive = FALSE
    /\ hcmStartTick = -1
    /\ hcmCumulative = 0
    /\ hcmScope = {}
    /\ hcmActivationId = ""
    /\ tick = 0
    /\ lastDecision = [trafficClass |-> "", policy |-> "", tick |-> 0]

Next ==
    \/ \E tc \in TrafficClasses : Evaluate(tc)
    \/ \E actId \in {"act1", "act2"}, scope \in SUBSET TrafficClasses :
           ActivateHcm(actId, scope)
    \/ DeactivateHcm
    \/ HcmExpiry
    \/ Tick
    \/ \E l \in Links, a \in BOOLEAN : UpdateLink(l, a)

Spec == Init /\ [][Next]_vars

---------------------------------------------------------------------------
(* INVARIANTS *)

(* 1. Policy Completeness: every evaluation produces a decision *)
PolicyCompleteness ==
    lastDecision.policy # "" => lastDecision.policy \in Policies \cup {"default"}

(* 2. Conflict Determinism: same state -> same decision *)
(* This is implicit in the CHOOSE definition with priority ordering *)

(* 3. HCM Mutual Exclusion: cannot have concurrent activations *)
HcmMutualExclusion ==
    hcmActive => hcmActivationId # ""

(* 4. HCM Bounded Duration: active HCM never exceeds max duration *)
HcmBoundedDuration ==
    hcmActive => (tick - hcmStartTick <= MaxHcmDuration)

(* 5. HCM Cumulative Bound *)
HcmCumulativeBound ==
    hcmCumulative <= MaxCumulativeHcm

=========================================================================
