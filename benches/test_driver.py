import unittest
from driver import command, select


class SelectionTests(unittest.TestCase):
    def test_ceiling_and_names(self):
        self.assertEqual([w[0] for w in select('Edit', '', '', [])], ['driver-tests'])
        self.assertEqual([w[0] for w in select('Change', 'parser', '', [])], ['parser'])
        with self.assertRaisesRegex(ValueError, 'no workloads'):
            select('Edit', 'parser', '', [])

    def test_invalid_selectors_and_empty_selection(self):
        for args in [('bogus', '', '', []), ('Change', 'bogus', '', []),
                     ('Change', '', 'bogus', []), ('Edit', '', '', ['driver-tests'])]:
            with self.assertRaises(ValueError):
                select(*args)

    def test_area_skip_and_busy_mode(self):
        chosen = select('Change', '', 'parser', ['parser'])
        self.assertTrue(chosen)
        self.assertTrue(all(w[2] == 'parser' and w[0] != 'parser' for w in chosen))
        self.assertIn('--test', command(chosen[0], True))
        self.assertNotIn('--test', command(chosen[0], False))
