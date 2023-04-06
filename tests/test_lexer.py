from helix import Lexer
from helix.helix_token import Keyword, TokenType


def test_operators():
    code = "(+ 1 2) / 3 * 4 - 5^2"

    tokens = list(Lexer(code).generate_tokens())

    assert len(tokens) == 14

    # test each token
    assert tokens[0].token_type == TokenType.LPAREN
    assert tokens[1].token_type == TokenType.PLUS
    assert tokens[2].token_type == TokenType.INT
    assert tokens[3].token_type == TokenType.INT
    assert tokens[4].token_type == TokenType.RPAREN
    assert tokens[5].token_type == TokenType.DIV
    assert tokens[6].token_type == TokenType.INT
    assert tokens[7].token_type == TokenType.MUL
    assert tokens[8].token_type == TokenType.INT
    assert tokens[9].token_type == TokenType.MINUS
    assert tokens[10].token_type == TokenType.INT
    assert tokens[11].token_type == TokenType.POW
    assert tokens[12].token_type == TokenType.INT
    assert tokens[13].token_type == TokenType.EOF


def test_keywords():
    code = "if 1 == 2 {\nlet x = 3\n}"

    tokens = list(Lexer(code).generate_tokens())

    assert len(tokens) == 13

    assert tokens[0].value == Keyword.IF
    assert tokens[1].token_type == TokenType.INT
    assert tokens[2].token_type == TokenType.EQ
    assert tokens[3].token_type == TokenType.INT
    assert tokens[4].token_type == TokenType.LBRACE
    assert tokens[5].token_type == TokenType.NEWLINE
    assert tokens[6].value == Keyword.LET
    assert tokens[7].token_type == TokenType.IDENTIFIER
    assert tokens[8].token_type == TokenType.ASSIGN
    assert tokens[9].token_type == TokenType.INT
    assert tokens[10].token_type == TokenType.NEWLINE
    assert tokens[11].token_type == TokenType.RBRACE
    assert tokens[12].token_type == TokenType.EOF
