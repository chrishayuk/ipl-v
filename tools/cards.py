"""Lay out IPL-V cards in the columns of the coding sheet:

    python3 tools/cards.py deck.src deck.iplv

The examples and the tutorial's decks were written this way. It is a
convenience for typing, not part of the language: the deck is the cards.

Source lines:
    #9 comment                      first card
    #1 comment                      comment card
    #2 A 10                         region A of 10 cells
    #5 routines | #5 data           header cards
    #5 start E1                     start card
    name | pq | symb | link | comment        a word (fields may be empty)
    name | +21 | HELLO || comment            sign before pq for data in routines
    !<card>                                  a card copied as it is
"""
import sys
def card(comment='', typ=' ', name='', sign=' ', pq='', symb='', link=''):
    if len(comment) > 40:
        raise SystemExit(f"a comment of {len(comment)} columns: {comment}")
    pq = pq.rjust(2) if pq else '  '
    s = comment[:40].ljust(40) + typ + ' ' + name.ljust(5)[:5] + sign + pq + symb.ljust(5)[:5] + ' ' + link.ljust(5)[:5]
    return s.rstrip()
def convert(text):
    out = []
    for line in text.split('\n'):
        if not line.strip():
            continue
        if line.startswith('!'):
            out.append(line[1:].rstrip())
            continue
        if line.startswith('#'):
            t = line[1]; rest = line[2:].strip()
            if t == '1':
                # A long comment is wrapped over several comment cards.
                import textwrap
                for piece in textwrap.wrap(rest.upper(), 40) or ['']:
                    out.append(card(piece, t))
            elif t == '9':
                out.append(card(rest.upper(), t))
            elif t == '2':
                r, n = rest.split()
                out.append(card('', '2', r, link=n.rjust(5)))
            elif t == '5':
                w = rest.split()
                if w[0] == 'routines': out.append(card('', '5', pq='00'))
                elif w[0] == 'data': out.append(card('', '5', pq='01'))
                elif w[0] == 'start': out.append(card('', '5', symb=w[1]))
            continue
        f = [x.strip(' ') if i != 2 else x for i, x in enumerate(line.split('|'))]
        f += [''] * (5 - len(f))
        name, pq, symb, link, comment = f[:5]
        if not pq.endswith('21'):
            symb = symb.strip()
        sign = ' '
        if pq[:1] in ('+', '-'):
            sign, pq = pq[0], pq[1:]
        out.append(card(comment.upper(), ' ', name, sign, pq, symb, link))
    return '\n'.join(out) + '\n'
if __name__ == '__main__':
    open(sys.argv[2], 'w').write(convert(open(sys.argv[1]).read()))
