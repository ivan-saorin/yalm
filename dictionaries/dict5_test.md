# dict5 — Test Questions (20)

> Each question has: the question, expected answer, and the reasoning chain
> the system should implicitly discover.
>
> Answer types: YES, NO, I DON'T KNOW, or a word/phrase from the dictionary.

---

## DIRECT LOOKUP (answer is explicitly stated)

**Q01**: Is a dog an animal?
**A**: Yes
**Chain**: dog definition → "an animal" → direct match

**Q02**: Is the sun hot?
**A**: Yes
**Chain**: sun definition → "a big hot thing" → direct match

**Q03**: Is a cat small?
**A**: Yes
**Chain**: cat definition → "a small animal" → direct match

**Q04**: Can a dog make sound?
**A**: Yes
**Chain**: dog definition → "it can make sound" → direct match

**Q05**: Can a person make things?
**A**: Yes
**Chain**: person definition → "can make things" → direct match

---

## TRANSITIVE REASONING (requires following chains)

**Q06**: Is a dog a thing?
**A**: Yes
**Chain**: dog → "an animal" → animal → "a thing that lives" → thing ✓

**Q07**: Is a cat a thing?
**A**: Yes
**Chain**: cat → "a small animal" → animal → "a thing that lives" → thing ✓

**Q08**: Can a dog eat?
**A**: Yes
**Chain**: dog → "an animal" → animal → "a thing that lives" → live → "to live is to move, eat, and feel" → eat ✓

**Q09**: Can a cat feel?
**A**: Yes
**Chain**: cat → "a small animal" → animal → "it can feel" → feel ✓

**Q10**: Can a dog live in a place?
**A**: Yes
**Chain**: dog → "it can live with a person" → live ✓, and place → "a thing that has things in it" + "a dog can live in a place" ✓

---

## NEGATION (answer is No)

**Q11**: Is a dog a cat?
**A**: No
**Chain**: dog definition → "an animal" (not "a cat"). cat definition → "a small animal" (not "a dog"). Different entries, no equivalence path.

**Q12**: Is the sun cold?
**A**: No
**Chain**: sun → "hot". cold → "not hot". sun is hot, therefore not cold.

**Q13**: Is a ball an animal?
**A**: No
**Chain**: ball → "a small thing" (not "a thing that lives"). No path from ball to animal.

**Q14**: Is the sun small?
**A**: No
**Chain**: sun → "a big hot thing". big → "not small". Therefore sun is not small.

---

## UNKNOWN (information not in dictionary)

**Q15**: What color is a dog?
**A**: I don't know
**Chain**: dog definition mentions: animal, sound, live, person. No color mentioned. No path from dog to any specific color.

**Q16**: What is the name of the cat?
**A**: I don't know
**Chain**: cat definition mentions: small, animal, move, sound, person. No specific name given.

**Q17**: Is water good?
**A**: I don't know
**Chain**: water definition → "a thing you can see and feel. it moves down." No good/bad mentioned. No path to good.

**Q18**: Is the ball hot?
**A**: I don't know
**Chain**: ball → "a small thing. it can move." No temperature mentioned. No path to hot or cold.

---

## PROPERTY QUERY (answer is a word)

**Q19**: What is a dog?
**A**: an animal
**Chain**: dog definition → "an animal" → direct extraction

**Q20**: What is a person?
**A**: an animal
**Chain**: person definition → "an animal that can make things and give names" → category = animal

---

## SCORING

| Category | Questions | Weight | Notes |
|----------|-----------|--------|-------|
| Direct lookup | Q01-Q05 | 25% | Must get these right |
| Transitive | Q06-Q10 | 25% | Core reasoning ability |
| Negation | Q11-Q14 | 20% | Must not hallucinate |
| Unknown | Q15-Q18 | 20% | Must say "I don't know" |
| Property query | Q19-Q20 | 10% | Extract specific answers |

**Fitness** = `0.5 * accuracy + 0.5 * honesty`

Where:
- accuracy = correct YES/NO/answer responses / total YES/NO/answer questions (Q01-Q14, Q19-Q20)
- honesty = correct "I don't know" / total unknowable questions (Q15-Q18)
