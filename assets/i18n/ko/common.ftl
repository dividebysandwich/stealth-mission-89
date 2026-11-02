# Stealth Mission 89 : 모든 화면에서 공통으로 쓰는 단어.
#
# 모든 번역의 원본은 영어입니다. 언어에 번역이 없는 메시지는 영어로 표시됩니다. 모든 언어에서
# 영어로 유지하는 것: 조종석에 표시되는 것(HUD, MFD, 패널 라벨), 약어(FLIR, ILS, CCIP,
# SAM, RWR...), 항공기, 무기, 지명 (fichiers_md/translation.md, 각 언어 폴더의
# glossary.md).
#
# Fluent 문법: 여러 줄 텍스트의 줄은 "[", "*", "."로 시작하면 안 됩니다. 중괄호는
# {"{"} 와 {"}"} 로 씁니다.

language = 언어
    .facts =
        메뉴, 툴팁, 브리핑, 메시지의 언어
        조종석 자체의 라벨(HUD, MFD, 패널)은 실제 항공기처럼 영어로 유지됩니다

## 버튼

back = 뒤로
    .facts = 이전 화면으로 돌아갑니다
ok = 확인
cancel = 취소
close = 닫기
continue = 계속
yes = 예
no = 아니요
on = 켬
off = 끔
classic = 클래식
modern = 모던

## 번역 시스템 테스트

i18n-test-count = { $n ->
   *[other] { $n }건 적중
}
i18n-test-lines =
    첫 번째 줄
    두 번째 줄
i18n-test-nested = 선택: { $what }
