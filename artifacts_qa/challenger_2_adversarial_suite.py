#!/usr/bin/env python3
import os, sys, json, time, random, sqlite3, tempfile, threading, traceback
from typing import Dict, List, Any, Tuple

REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
if REPO_ROOT not in sys.path: sys.path.insert(0, REPO_ROOT)

from tools.studylab_content_factory import (
    get_all_175_topics,
    build_apkg_from_topics,
    validate_all_contracts,
    NOTETYPE_NAME
)

print('[1/4] Starting Challenger 2 Adversarial Verification...')
