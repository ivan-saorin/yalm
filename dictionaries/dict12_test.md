# dict12 — Test Questions (20)

> Each question has: the question, expected answer, and the reasoning chain
> the system should implicitly discover.
>
> Answer types: YES, NO, I DON'T KNOW, or a word/phrase from the dictionary.

---

## DIRECT LOOKUP (answer is explicitly stated)

**Q01**: Is a dog a mammal?
**A**: Yes
**Chain**: dog definition → "a domestic mammal" → direct match

**Q02**: Is the sun a star?
**A**: Yes
**Chain**: sun definition → "the star at the center" → direct match

**Q03**: Is water a liquid?
**A**: Yes
**Chain**: water definition → "a clear liquid" → direct match

**Q04**: Can a cat climb?
**A**: Yes
**Chain**: cat examples → "the cat climbed" → direct match

**Q05**: Does food give energy?
**A**: Yes
**Chain**: food definition → "to get energy" → direct match

---

## TRANSITIVE REASONING (requires following chains)

**Q06**: Is a dog an animal?
**A**: Yes
**Chain**: dog → "a domestic mammal" → mammal → "an animal" → animal ✓ (2 hops)

**Q07**: Is a cat an animal?
**A**: Yes
**Chain**: cat → "a small domestic mammal" → mammal → "an animal" → animal ✓ (2 hops)

**Q08**: Is a wolf an animal?
**A**: Yes
**Chain**: wolf → "a wild animal" → animal ✓ (1 hop, uses closure word)

**Q09**: Does a plant need water?
**A**: Yes
**Chain**: plant examples → "plants need water" → water ✓

**Q10**: Does a dog need food?
**A**: Yes
**Chain**: dog → "a domestic mammal" → mammal → "an animal" → animal → "needs food" → food ✓ (3 hops)

---

## NEGATION (answer is No)

**Q11**: Is a plant an animal?
**A**: No
**Chain**: animal definition → "not a plant". Explicit negation — plant is excluded from animal.

**Q12**: Is a wolf domestic?
**A**: No
**Chain**: wolf → "a wild animal". wild → "not domestic". Therefore wolf is not domestic.

**Q13**: Is ice hot?
**A**: No
**Chain**: ice → "frozen water" → cold. cold → "not hot". Therefore ice is not hot.

**Q14**: Is a rock alive?
**A**: No
**Chain**: rock → "a solid object". No path from rock to alive/living. Objects are not alive.

---

## UNKNOWN (information not in dictionary)

**Q15**: What is the name of the sun?
**A**: I don't know
**Chain**: sun definition mentions: star, center, solar system, light, heat. No specific name given.

**Q16**: What color is a dog?
**A**: I don't know
**Chain**: dog definition mentions: domestic, mammal, loyal, companion. No color mentioned. No path from dog to any specific color.

**Q17**: Is a mountain good?
**A**: I don't know
**Chain**: mountain definition → landmass, peak, elevation. No good/bad mentioned. No path to good.

**Q18**: Can a ball think?
**A**: I don't know
**Chain**: ball → "a round object". think → "to use the mind". No path from ball to think/mind.

---

## PROPERTY QUERY (answer is a word)

**Q19**: What is a cat?
**A**: a mammal
**Chain**: cat definition → "a small domestic mammal" → category = mammal

**Q20**: What is a wolf?
**A**: an animal
**Chain**: wolf definition → "a wild animal" → category = animal

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
