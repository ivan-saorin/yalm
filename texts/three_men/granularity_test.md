# granularity_test — Coarse-to-Fine Comprehension Probe

## LEVEL 1 — ONTOLOGICAL

---

**Q01**: Is Montmorency a thing?
**A**: Yes
**Chain**: montmorency -> dog -> animal -> thing (3-hop transitive)

---

**Q02**: Is Harris a thing?
**A**: Yes
**Chain**: harris -> person -> human -> thing (3-hop)

---

**Q03**: Is the Thames a thing?
**A**: Yes
**Chain**: thames -> river -> water -> thing (3-hop)

---

**Q04**: Is Kingston a thing?
**A**: Yes
**Chain**: kingston -> place -> thing (2-hop)

---

**Q05**: Is Montmorency alive?
**A**: Yes
**Chain**: montmorency -> dog -> animal -> alive (3-hop, alive="a state. a thing has life")

---

**Q06**: Is the Thames alive?
**A**: No
**Chain**: thames -> river (river is not alive)

---

**Q07**: Is Harris alive?
**A**: Yes
**Chain**: harris -> person -> human -> alive (3-hop)

---

**Q08**: Is a dog a thing?
**A**: Yes
**Chain**: dog -> animal -> thing (2-hop)

---

## LEVEL 2 — KINGDOM

---

**Q09**: Is Montmorency a place?
**A**: No
**Chain**: montmorency -> place

---

**Q10**: Is the Thames an animal?
**A**: No
**Chain**: thames -> animal

---

**Q11**: Is Kingston a person?
**A**: No
**Chain**: kingston -> person

---

**Q12**: Is Harris a place?
**A**: No
**Chain**: harris -> place

---

**Q13**: Is Hampton a person?
**A**: No
**Chain**: hampton -> person

---

**Q14**: Is Hampton an animal?
**A**: No
**Chain**: hampton -> animal

---

## LEVEL 3 — SPECIES/TYPE

---

**Q15**: Is Montmorency a fox terrier?
**A**: Yes
**Chain**: montmorency -> terrier (entity def: "a small fox terrier")

---

**Q16**: Is Harris a man?
**A**: Yes
**Chain**: harris -> man (entity def: "he is a man")

---

**Q17**: Is George a man?
**A**: Yes
**Chain**: george -> man (entity def: "he is a man")

---

**Q18**: Is Kingston a town?
**A**: Yes
**Chain**: kingston -> town (entity def: "a town on the thames")

---

**Q19**: Is the Thames a big river?
**A**: Yes
**Chain**: thames -> big (entity def: "a big river")

---

**Q20**: What is a terrier?
**A**: a dog
**Chain**: terrier -> dog (ollama def: "a dog")

---

## LEVEL 4 — PROPERTIES & CAPABILITIES

---

**Q21**: Can a dog move?
**A**: Yes
**Chain**: dog -> move (ollama def: "it can make sound and move fast")

---

**Q22**: Can a person think?
**A**: Yes
**Chain**: person -> think (ollama def: "it can think and talk")

---

**Q23**: Can a person talk?
**A**: Yes
**Chain**: person -> talk (ollama def: "it can think and talk")

---

**Q24**: Can an animal eat?
**A**: Yes
**Chain**: animal -> eat (ollama def: "it can move and eat")

---

**Q25**: Can an animal move?
**A**: Yes
**Chain**: animal -> move (ollama def: "it can move and eat")

---

**Q26**: Is a dog big?
**A**: I don't know
**Chain**: dog -> big (dogs vary in size, not in def)

---

**Q27**: Is the Thames big?
**A**: Yes
**Chain**: thames -> big (entity def: "a big river")

---

**Q28**: Is a town a place?
**A**: Yes
**Chain**: town -> place (ollama def: "a place")

---

**Q29**: Is a maze a place?
**A**: Yes
**Chain**: maze -> place (ollama def: "a place with many paths")

---

**Q30**: Can a river move?
**A**: Yes
**Chain**: river -> move (ollama def: "it can move")

---

## LEVEL 5 — RELATIONAL

---

**Q31**: Is Kingston on the Thames?
**A**: Yes
**Chain**: kingston -> thames (entity def: "a town on the thames river")

---

**Q32**: Is Hampton near the Thames?
**A**: Yes
**Chain**: hampton -> thames (entity def: "near the thames")

---

**Q33**: Is Harris on the boat?
**A**: Yes
**Chain**: harris -> boat (entity def: "on the boat trip")

---

**Q34**: Is George on the boat?
**A**: Yes
**Chain**: george -> boat (entity def: "on the boat trip")

---

**Q35**: Is Montmorency on the boat?
**A**: Yes
**Chain**: montmorency -> boat (entity def: "goes on the boat trip")

---

**Q36**: Is Kingston near Hampton?
**A**: I don't know
**Chain**: kingston -> hampton (not stated in definitions)

---

**Q37**: Is the Thames in Kingston?
**A**: I don't know
**Chain**: thames -> kingston (reversed relationship)

---

**Q38**: Is Harris with George?
**A**: I don't know
**Chain**: harris -> george (implied but not stated)

---

**Q39**: Is a boat on a river?
**A**: I don't know
**Chain**: boat -> river (ollama def: "floats on water" not "on river")

---

**Q40**: Is Hampton a building?
**A**: Yes
**Chain**: hampton -> building (entity def: "a big old building")

---

## LEVEL 6 — NARRATIVE CHARACTERIZATION

---

**Q41**: Is Montmorency small?
**A**: Yes
**Chain**: montmorency -> small (entity def: "a small fox terrier")

---

**Q42**: Is Harris a friend?
**A**: I don't know
**Chain**: harris -> friend (narrative implies, not in entity def)

---

**Q43**: Is the Thames long?
**A**: I don't know
**Chain**: thames -> long (not stated in entity def)

---

**Q44**: Can Montmorency fight?
**A**: I don't know
**Chain**: montmorency -> fight (narrative yes, depends on def chain)

---

**Q45**: Can a dog run?
**A**: Yes
**Chain**: dog -> run (ollama def: "move fast" -> run is close)

---

**Q46**: Can a dog eat?
**A**: Yes
**Chain**: dog -> eat (dog -> animal -> "can eat")

---

**Q47**: Is Hampton old?
**A**: Yes
**Chain**: hampton -> old (entity def: "a big old building")

---

**Q48**: Can a person walk?
**A**: I don't know
**Chain**: person -> walk (not directly in person def, maybe via chain)

---

**Q49**: Is a fox an animal?
**A**: Yes
**Chain**: fox -> animal (ollama def: "a small animal")

---

**Q50**: Can a person feel?
**A**: Yes
**Chain**: person -> feel (ollama def: "it has feelings")
