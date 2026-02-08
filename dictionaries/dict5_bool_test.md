# dict5_bool — Boolean Operator Tests (10)

## AND — both must be Yes

---

**Q01**: Is a dog an animal and a thing?
**A**: Yes
**Chain**: dog→animal=Yes AND dog→animal→thing=Yes → AND→Yes

---

**Q02**: Is a dog an animal and a cat?
**A**: No
**Chain**: dog→animal=Yes AND dog→cat=No → AND→No

---

**Q03**: Is the sun big and hot?
**A**: Yes
**Chain**: sun→big=Yes AND sun→hot=Yes → AND→Yes

---

**Q04**: Is the sun hot and cold?
**A**: No
**Chain**: sun→hot=Yes AND sun→cold=No → AND→No

---

**Q05**: Is a ball an animal and a thing?
**A**: No
**Chain**: ball→animal=No AND ball→thing=Yes → AND→No

---

## OR — either can be Yes

---

**Q06**: Is a dog a cat or an animal?
**A**: Yes
**Chain**: dog→cat=No OR dog→animal=Yes → OR→Yes

---

**Q07**: Is the sun hot or cold?
**A**: Yes
**Chain**: sun→hot=Yes OR sun→cold=No → OR→Yes

---

**Q08**: Is a cat a dog or a ball?
**A**: No
**Chain**: cat→dog=No OR cat→ball=No → OR→No

---

**Q09**: Is a dog an animal or a person?
**A**: Yes
**Chain**: dog→animal=Yes OR dog→person=No → OR→Yes

---

**Q10**: Can a dog eat and move?
**A**: Yes
**Chain**: dog→eat=Yes AND dog→move=Yes → AND→Yes
