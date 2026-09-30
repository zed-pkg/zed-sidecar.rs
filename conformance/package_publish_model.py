#!/usr/bin/env python3
from itertools import product

def main():
    checked=0
    for digest_locked,signature_ok,name_bound,publish in product((False,True),repeat=4):
        admitted=(not publish) or (digest_locked and signature_ok and name_bound)
        if publish and admitted:
            assert digest_locked,'publish admitted with mutable digest'
            assert signature_ok,'publish admitted without valid signature'
            assert name_bound,'publish admitted without package-name binding'
        checked+=1
    print(f'package publish model: {checked} states')
if __name__=='__main__':main()
