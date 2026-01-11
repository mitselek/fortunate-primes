#!/bin/bash
# Resume Fortunate primes expedition for n=4639 from checkpoint
# Created: 2026-01-11

cd ~/Documents/github/mitselek/projects/fortunate-primes/implementations/python-gmpy2
source .venv/bin/activate
python fortunate_expedition.py 4639 4639 --resume --log expedition_4639.md
