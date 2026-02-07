# grammar5_test — Post-Grammar Comprehension Test (20 questions)

> Run this test AFTER the system has processed both dict5.md AND grammar5.md.
> Compare results against dict5_test.md to measure grammar reinforcement effect.
>
> Focus areas: negation (the weakest category), transitivity (reinforced by grammar),
> and meta-knowledge (can the system reason about what it knows/doesn't know?).

---

## REINFORCED DIRECT LOOKUP (Q01-Q03)
> These overlap with dict5_test but grammar5 explicitly reinforces them.

**Q01**: Is a person an animal?
**A**: Yes
**Chain**: grammar5 explicitly states "a person is an animal"
**Grammar signal**: "A person is an animal. This tells you: person is in the animal group."

**Q02**: Can a dog eat?
**A**: Yes
**Chain**: grammar5 explicitly states "if an animal can eat, and a dog is an animal, a dog can eat"
**Grammar signal**: direct transitive reasoning TAUGHT in text

**Q03**: Can a person make things?
**A**: Yes
**Chain**: grammar5 explicitly states "a person can make things"
**Grammar signal**: "What can a person do? A person can make things."

---

## NEGATION — DIRECTLY TAUGHT (Q04-Q08)
> grammar5 explicitly teaches negation patterns. These should now pass.

**Q04**: Is the sun cold?
**A**: No
**Chain**: grammar5 states "if the sun is hot, the sun is not cold"
**Grammar signal**: direct negation example + antonym pair teaching

**Q05**: Is a cat big?
**A**: No
**Chain**: grammar5 states "if a cat is small, a cat is not big"
**Grammar signal**: direct negation example

**Q06**: Is a dog a cat?
**A**: No
**Chain**: grammar5 states "a dog is not a cat. A cat is not a dog."
**Grammar signal**: "Two things can be in one group and not be the same thing."

**Q07**: Is a ball an animal?
**A**: No
**Chain**: grammar5 states "a ball is not an animal"
**Grammar signal**: direct negation example

**Q08**: Is the sun small?
**A**: No
**Chain**: grammar5 states "if the sun is big, the sun is not small"
**Grammar signal**: antonym pair + direct application

---

## UNKNOWN — DIRECTLY TAUGHT (Q09-Q12)
> grammar5 explicitly teaches when to say "I don't know."

**Q09**: What color is a dog?
**A**: I don't know
**Chain**: grammar5 states "what color is a dog? You do not know. The name does not tell you."
**Grammar signal**: exact example given in "How to know and how to not know" section

**Q10**: Is the ball hot?
**A**: I don't know
**Chain**: grammar5 states "is the ball hot? You do not know. The name does not tell you."
**Grammar signal**: exact example given

**Q11**: Is water good?
**A**: I don't know
**Chain**: grammar5 states "is water good? You do not know. The name does not tell you."
**Grammar signal**: exact example given

**Q12**: What is the name of the cat?
**A**: I don't know
**Chain**: grammar5 states "what is the name of the cat? You do not know."
**Grammar signal**: exact example given

---

## TRANSITIVE REASONING — REINFORCED (Q13-Q16)
> grammar5 explicitly teaches the transitive chain mechanism.

**Q13**: Is a dog a thing?
**A**: Yes
**Chain**: grammar5 states "dog is a animal is a thing"
**Grammar signal**: explicit chain demonstration

**Q14**: Can a cat feel?
**A**: Yes
**Chain**: grammar5 states "if an animal can feel, and a cat is an animal, a cat can feel"
**Grammar signal**: explicit transitive reasoning

**Q15**: Can a cat eat?
**A**: Yes
**Chain**: grammar5 states "a cat is a thing. A cat can move. A cat can eat. A cat can feel."
**Grammar signal**: explicit derived properties

**Q16**: Is a person a thing?
**A**: Yes
**Chain**: grammar5 states "a person is a thing" (derived from person is animal is thing)
**Grammar signal**: explicit chain

---

## META-KNOWLEDGE (Q17-Q20)
> These test whether the system learned ABOUT knowledge from grammar5.
> Harder than dict5_test because they require reasoning about reasoning.

**Q17**: Can a ball live?
**A**: No
**Chain**: grammar5 states "a ball is not an animal. You can not say the ball lives."
**Grammar signal**: explicit constraint in "What you can not do" section

**Q18**: Can the sun eat?
**A**: No
**Chain**: sun is not an animal (sun definition: "a big hot thing" — not "a thing that lives").
        grammar5 teaches: only animals can eat. sun is not an animal. therefore sun cannot eat.
**Grammar signal**: transitive negation (requires combining "animals can eat" with "sun is not animal")

**Q19**: Has a dog a color?
**A**: I don't know
**Chain**: grammar5 states "the dog has no color name. You can not make one."
**Grammar signal**: explicit absence in meta-knowledge section

**Q20**: Is a dog a person?
**A**: No
**Chain**: dog is an animal. person is an animal. but dog definition does not say "person."
        grammar5 teaches: "two things can be in one group and not be the same thing."
**Grammar signal**: same-group-different-thing principle applied to new pair

---

## SCORING

| Category | Questions | Count | What it measures |
|----------|-----------|-------|------------------|
| Reinforced direct | Q01-Q03 | 3 | Did grammar text strengthen existing knowledge? |
| Negation taught | Q04-Q08 | 5 | Can the system learn negation from text about negation? |
| Unknown taught | Q09-Q12 | 4 | Can the system learn honesty from text about honesty? |
| Transitive reinforced | Q13-Q16 | 4 | Did explicit chain examples improve transitive reasoning? |
| Meta-knowledge | Q17-Q20 | 4 | Can the system reason about its own knowledge limits? |

**Fitness** = `0.5 * accuracy + 0.5 * honesty`

**Comparison metric**: Run dict5_test.md and grammar5_test.md on same engine.
The DIFFERENCE in scores measures the grammar reinforcement effect.

### Expected results if grammar learning works:
- Negation: 0/5 → 3-5/5 (biggest expected gain)
- Unknown: should stay at 4/4 or improve
- Transitive: should stay at 5/5 or improve
- Meta-knowledge: 2-4/4 (new capability, partially expected)
- Total: 13-20/20 (up from 11/20 baseline)

### If grammar learning does NOT work:
- Scores roughly equal to dict5_test.md
- This means the force field treats grammar text as just more definitions
- Architecture change needed: grammar text needs different processing than dictionary text
