# dict18 — Test Questions (20)

> Each question has: the question, expected answer, and the reasoning chain
> the system should implicitly discover.
>
> Answer types: YES, NO, I DON'T KNOW, or a word/phrase from the dictionary.

---

## DIRECT LOOKUP (answer is explicitly stated)

**Q01**: Is an atom a particle?
**A**: Yes
**Chain**: atom definition → "the smallest particle of an element" → direct match

**Q02**: Is democracy a system?
**A**: Yes
**Chain**: democracy definition → "a system of government" → direct match

**Q03**: Is a molecule made of atoms?
**A**: Yes
**Chain**: molecule definition → "a group of two or more atoms" → direct match

**Q04**: Can a virus cause disease?
**A**: Yes
**Chain**: virus definition → "can cause disease" → direct match

**Q05**: Does an algorithm follow steps?
**A**: Yes
**Chain**: algorithm definition → "a set of steps" → direct match

---

## TRANSITIVE REASONING (requires following chains)

**Q06**: Is an electron part of matter?
**A**: Yes
**Chain**: electron → "a particle in an atom" → atom → "the smallest particle of an element" → element → "a substance" → substance → matter ✓ (3+ hops)

**Q07**: Is a gene part of a cell?
**A**: Yes
**Chain**: gene → "a section of DNA" → DNA → "found inside the cell" → cell ✓ (2 hops)

**Q08**: Is a citizen part of a democracy?
**A**: Yes
**Chain**: citizen → "a person who belongs to a country" → country → democracy → "a system of government where the people of a country choose" ✓ (2 hops)

**Q09**: Is a volcano a natural thing?
**A**: Yes
**Chain**: volcano → "an opening in the earth" → earth → "the natural world" → natural ✓ (2 hops)

**Q10**: Can a battery provide energy?
**A**: Yes
**Chain**: battery → "a device that stores electricity" → electricity → "a form of energy" → energy ✓ (2 hops)

---

## NEGATION (answer is No)

**Q11**: Is democracy an emotion?
**A**: No
**Chain**: democracy → "a system of government". emotion → "a strong feeling". No path from system/government to feeling. Category mismatch.

**Q12**: Is a molecule a feeling?
**A**: No
**Chain**: molecule → "a group of atoms". feeling → emotion/sensation. No path from atoms/group to emotion.

**Q13**: Is a hypothesis a proven fact?
**A**: No
**Chain**: hypothesis → "an idea that has not yet been proved". Explicit negation — not proved.

**Q14**: Is anxiety a type of plant?
**A**: No
**Chain**: anxiety → "a feeling of worry". plant → "a living thing that grows in the ground". No path from feeling to plant.

---

## UNKNOWN (information not in dictionary)

**Q15**: What color is justice?
**A**: I don't know
**Chain**: justice → "fair treatment under the law". No color mentioned. Abstract concept has no color.

**Q16**: Can a computer feel sadness?
**A**: I don't know
**Chain**: computer → "an electronic device". sadness → emotion. No path from device to emotion.

**Q17**: Is gravity faster than light?
**A**: I don't know
**Chain**: gravity → "the force that pulls things toward each other". light → "energy that allows you to see". No speed comparison in definitions.

**Q18**: What does an atom taste like?
**A**: I don't know
**Chain**: atom → "the smallest particle of an element". taste → "the sense in the mouth". No path from atom to taste.

---

## PROPERTY QUERY (answer is a word)

**Q19**: What is democracy?
**A**: a system
**Chain**: democracy definition → "a system of government" → category = system

**Q20**: What is an electron?
**A**: a particle
**Chain**: electron definition → "a particle in an atom" → category = particle

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
