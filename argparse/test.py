import argparse

parser = argparse.ArgumentParser("git", description="A finctional versioning CLI")
subps = parser.add_subparsers()

clone = subps.add_parser("clone", description="Clones repos")
clone.add_argument("remote", help="The remote to clone", metavar="<REMOTE>")

diff = subps.add_parser("diff", description="Compare two commits")
diff.add_argument("base", metavar="[COMMIT]")
diff.add_argument("head", metavar="[COMMIT]")
diff.add_argument("path", metavar="[PATH]")
diff.add_argument(
    "--colour",
    metavar="<WHEN>",
    choices=("always", "auto", "never"),
    nargs=1,
    default="auto"
)

test = subps.add_parser("test")
test.add_argument("pos1", action='store_const', const="asdf")
test.add_argument("-o", action='store', nargs=3)

print(parser.parse_args())


# parser = argparse.ArgumentParser()
# parser.add_argument('--foo', action='append', default=['0'])
# print(parser.parse_args('--foo 1 --foo 2 --foo 3 4'.split()))
